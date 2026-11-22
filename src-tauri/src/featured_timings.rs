use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

const MANAGER_URL: &str = "http://82.64.40.23:33255";
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Deserialize)]
struct Leaderboard {
    #[serde(
        rename = "DisconnectedDrivers",
        deserialize_with = "Deserialize::deserialize"
    )]
    disconnected_drivers: Option<Vec<Driver>>,
}

#[derive(Deserialize)]
struct Driver {
    #[serde(rename = "CarInfo")]
    car_info: CarInfo,
    #[serde(rename = "Cars")]
    cars: HashMap<String, Car>,
}

#[derive(Deserialize)]
struct CarInfo {
    #[serde(rename = "DriverName")]
    driver_name: String,
}

#[derive(Deserialize)]
struct Car {
    #[serde(rename = "BestLap")]
    best_lap: i64,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct BestStoredLap {
    driver: String,
    milliseconds: u64,
}

fn parse_leaderboard(body: &[u8]) -> Result<Option<BestStoredLap>, String> {
    let leaderboard: Leaderboard = serde_json::from_slice(body)
        .map_err(|error| format!("Réponse des chronos invalide : {error}"))?;
    let mut best: Option<(i64, String)> = None;
    for driver in leaderboard.disconnected_drivers.unwrap_or_default() {
        for car in driver.cars.values() {
            if car.best_lap < 1_000_000 {
                continue;
            }
            if driver.car_info.driver_name.trim().is_empty() {
                return Err("Nom de pilote manquant dans les chronos.".into());
            }
            if best.as_ref().is_none_or(|(time, _)| car.best_lap < *time) {
                best = Some((car.best_lap, driver.car_info.driver_name.clone()));
            }
        }
    }
    Ok(best.map(|(time, driver)| BestStoredLap {
        driver,
        milliseconds: (time / 1_000_000) as u64,
    }))
}

async fn fetch_best_stored_lap(
    client: &reqwest::Client,
    base_url: &str,
    server_index: u8,
) -> Result<Option<BestStoredLap>, String> {
    let selection = client
        .get(format!("{base_url}/live-timing?server={server_index}"))
        .send()
        .await
        .map_err(|error| format!("Sélection du serveur impossible : {}", error.without_url()))?;
    if !selection.status().is_success() {
        return Err(format!(
            "Sélection du serveur : HTTP {}.",
            selection.status()
        ));
    }
    let cookie = selection
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .filter_map(|header| header.split(';').next())
        .find(|cookie| cookie.starts_with("current-server="))
        .ok_or("Cookie de sélection du serveur manquant.")?;
    let mut response = client
        .get(format!("{base_url}/api/live-timings/leaderboard.json"))
        .header(reqwest::header::COOKIE, cookie)
        .send()
        .await
        .map_err(|error| {
            format!(
                "Chargement des chronos impossible : {}",
                error.without_url()
            )
        })?;
    if !response.status().is_success() {
        return Err(format!(
            "Chargement des chronos : HTTP {}.",
            response.status()
        ));
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_RESPONSE_BYTES as u64)
    {
        return Err("La réponse des chronos est trop volumineuse.".into());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Lecture des chronos impossible : {}", error.without_url()))?
    {
        if chunk.len() > MAX_RESPONSE_BYTES - body.len() {
            return Err("La réponse des chronos est trop volumineuse.".into());
        }
        body.extend_from_slice(&chunk);
    }
    parse_leaderboard(&body)
}

#[tauri::command]
pub(crate) async fn get_featured_stored_lap(
    server_index: u8,
) -> Result<Option<BestStoredLap>, String> {
    if server_index > 2 {
        return Err("Serveur d'essais libres invalide.".into());
    }
    let client = reqwest::Client::builder()
        .user_agent("QARL")
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| format!("Impossible de créer le client chronos : {error}"))?;
    fetch_best_stored_lap(&client, MANAGER_URL, server_index).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_fastest_stored_lap_across_drivers_and_cars() {
        let body = br#"{"DisconnectedDrivers":[
            {"CarInfo":{"DriverName":"Alice"},"Cars":{"a":{"BestLap":125251000000},"b":{"BestLap":0}}},
            {"CarInfo":{"DriverName":"Bob"},"Cars":{"a":{"BestLap":121658000000},"b":{"BestLap":116856000000}}}
        ]}"#;
        assert_eq!(
            parse_leaderboard(body).unwrap(),
            Some(BestStoredLap {
                driver: "Bob".into(),
                milliseconds: 116856,
            })
        );
    }

    #[test]
    fn distinguishes_no_stored_laps_from_invalid_data() {
        assert_eq!(
            parse_leaderboard(br#"{"DisconnectedDrivers":null}"#).unwrap(),
            None
        );
        assert_eq!(
            parse_leaderboard(br#"{"DisconnectedDrivers":[]}"#).unwrap(),
            None
        );
        assert!(parse_leaderboard(br#"{}"#).is_err());
        assert!(parse_leaderboard(br#"{"DisconnectedDrivers":[{"CarInfo":{"DriverName":""},"Cars":{"a":{"BestLap":1000000000}}}]}"#).is_err());
    }

    #[tokio::test]
    async fn selects_server_before_requesting_its_leaderboard() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            for index in 0..2 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                let mut chunk = [0; 512];
                while !request.ends_with(b"\r\n\r\n") && request.len() < 4096 {
                    let count = socket.read(&mut chunk).await.unwrap();
                    assert_ne!(count, 0);
                    request.extend_from_slice(&chunk[..count]);
                }
                let request = std::str::from_utf8(&request).unwrap();
                let response = if index == 0 {
                    assert!(request.starts_with("GET /live-timing?server=2 "));
                    "HTTP/1.1 200 OK\r\nSet-Cookie: current-server=signed-value; Path=/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned()
                } else {
                    assert!(request.starts_with("GET /api/live-timings/leaderboard.json "));
                    assert!(
                        request
                            .to_ascii_lowercase()
                            .contains("cookie: current-server=signed-value\r\n")
                    );
                    let body = r#"{"DisconnectedDrivers":[{"CarInfo":{"DriverName":"Bob"},"Cars":{"a":{"BestLap":116856000000}}}]}"#;
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                };
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let lap = fetch_best_stored_lap(&client, &format!("http://{address}"), 2)
            .await
            .unwrap();
        server.await.unwrap();
        assert_eq!(
            lap,
            Some(BestStoredLap {
                driver: "Bob".into(),
                milliseconds: 116856
            })
        );
    }
}
