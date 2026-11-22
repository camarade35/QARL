use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Deserialize)]
struct SlotsResponse {
    #[serde(rename = "Cars")]
    cars: Vec<Slot>,
}

#[derive(Deserialize)]
struct Slot {
    #[serde(rename = "Model")]
    model: Option<String>,
    #[serde(rename = "IsConnected")]
    connected: Option<bool>,
    #[serde(rename = "IsEntryList")]
    entry_list: Option<bool>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct SlotCount {
    free: u32,
    total: u32,
}

fn parse_slots(body: &[u8]) -> Result<HashMap<String, SlotCount>, String> {
    let response: SlotsResponse = serde_json::from_slice(body)
        .map_err(|error| format!("Réponse des places invalide : {error}"))?;
    let mut counts = HashMap::new();
    for slot in response.cars {
        let is_entry = slot.entry_list.ok_or("Statut d'entrée manquant pour une place.")?;
        if !is_entry {
            continue;
        }
        let model = slot.model.filter(|model| !model.trim().is_empty())
            .ok_or("Modèle manquant pour une place.")?.to_ascii_lowercase();
        let connected = slot.connected.ok_or("Statut de connexion manquant pour une place.")?;
        let count = counts.entry(model).or_insert(SlotCount { free: 0, total: 0 });
        count.total += 1;
        if !connected {
            count.free += 1;
        }
    }
    Ok(counts)
}

fn slots_url(ip: &str, port: u16) -> Result<String, String> {
    if port == 0 {
        return Err("Port HTTP invalide.".into());
    }
    let host = match ip.parse::<IpAddr>() {
        Ok(IpAddr::V6(address)) => format!("[{address}]"),
        Ok(IpAddr::V4(address)) => address.to_string(),
        Err(_) if !ip.is_empty()
            && ip.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-') => ip.to_owned(),
        Err(_) => return Err("Adresse du serveur invalide.".into()),
    };
    Ok(format!("http://{host}:{port}/JSON%7C-1"))
}

#[tauri::command]
pub(crate) async fn get_server_car_slots(ip: String, http_port: u16) -> Result<HashMap<String, SlotCount>, String> {
    let url = slots_url(&ip, http_port)?;
    let client = reqwest::Client::builder()
        .user_agent("Assetto Corsa Launcher")
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|error| format!("Impossible de créer le client serveur : {error}"))?;

    let mut response = client.get(url).send().await
        .map_err(|error| format!("Impossible de contacter le serveur : {}", error.without_url()))?;
    if !response.status().is_success() {
        return Err(format!("Le serveur a répondu avec le statut HTTP {}.", response.status()));
    }
    const LIMIT: usize = 4 * 1024 * 1024;
    if response.content_length().is_some_and(|size| size > LIMIT as u64) {
        return Err("La réponse du serveur est trop volumineuse.".into());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await
        .map_err(|error| format!("Lecture de la réponse impossible : {}", error.without_url()))?
    {
        if chunk.len() > LIMIT - body.len() {
            return Err("La réponse du serveur est trop volumineuse.".into());
        }
        body.extend_from_slice(&chunk);
    }
    parse_slots(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_free_entry_list_slots_by_model() {
        let response = br#"{"Cars":[
            {"Model":"Car_A","IsConnected":false,"IsEntryList":true},
            {"Model":"car_a","IsConnected":true,"IsEntryList":true},
            {"Model":"car_b","IsConnected":true,"IsEntryList":true},
            {"Model":"car_b","IsConnected":false,"IsEntryList":false}
        ]}"#;
        let counts = parse_slots(response).unwrap();
        assert_eq!(counts["car_a"], SlotCount { free: 1, total: 2 });
        assert_eq!(counts["car_b"], SlotCount { free: 0, total: 1 });
    }

    #[test]
    fn rejects_incomplete_or_invalid_slot_data() {
        assert!(parse_slots(br#"{"Cars":[{"Model":"x","IsEntryList":true}]}"#).is_err());
        assert!(parse_slots(br#"{"Cars":[{"Model":"x","IsConnected":false}]}"#).is_err());
        assert!(parse_slots(br#"{"Cars":[{"Model":"","IsEntryList":true,"IsConnected":false}]}"#).is_err());
        assert!(parse_slots(b"not json").is_err());
    }

    #[test]
    fn uses_anonymous_request_and_validates_address() {
        assert_eq!(slots_url("192.0.2.1", 8081).unwrap(), "http://192.0.2.1:8081/JSON%7C-1");
        assert_eq!(slots_url("::1", 8081).unwrap(), "http://[::1]:8081/JSON%7C-1");
        assert!(slots_url("example.com/other", 8081).is_err());
        assert!(slots_url("example.com", 0).is_err());
    }
}
