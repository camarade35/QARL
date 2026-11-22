use serde::{Deserialize, Serialize};

const LOBBY_URL: &str = "https://lobby.assettocorsa.net/lobby.ashx/list";
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Deserialize)]
struct LobbyServer {
    ip: String,
    cport: u16,
    port: Option<u16>,
    tport: Option<u16>,
    name: Option<String>,
    track: Option<String>,
    clients: Option<u32>,
    maxclients: Option<u32>,
    cars: Option<Vec<String>>,
    country: Option<Vec<String>>,
    pass: Option<bool>,
}

#[derive(Debug, Serialize)]
pub(crate) struct KunosServer {
    ip: String,
    http_port: u16,
    udp_port: Option<u16>,
    game_port: Option<u16>,
    name: Option<String>,
    track: Option<String>,
    players: Option<u32>,
    max_players: Option<u32>,
    cars: Option<Vec<String>>,
    country: Option<String>,
    password_required: Option<bool>,
}

fn parse_lobby_response(body: &[u8]) -> Result<Vec<KunosServer>, String> {
    if body.starts_with(b"ERROR,SLOW DOWN") {
        return Err("Le lobby Kunos limite les requêtes. Réessaie plus tard.".into());
    }

    let entries: Vec<LobbyServer> = serde_json::from_slice(body)
        .map_err(|error| format!("Réponse du lobby Kunos invalide : {error}"))?;

    if entries.len() > 1 {
        let first = &entries[0].ip;
        if first.parse::<std::net::Ipv4Addr>().is_ok_and(|ip| ip.is_private())
            && entries.iter().all(|entry| entry.ip == *first)
        {
            return Err("Le lobby Kunos a renvoyé une liste invalide. Vérifie que le Steam ID est associé à Assetto Corsa.".into());
        }
    }

    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            if entry.ip.parse::<std::net::IpAddr>().is_err() || entry.cport == 0 {
                return Err(format!("Adresse invalide pour le serveur Kunos n°{}.", index + 1));
            }

            let country = entry.country.and_then(|parts| {
                parts.get(1).filter(|code| {
                    code.len() == 2 && code.chars().all(|c| c.is_ascii_alphabetic())
                }).map(|code| code.to_ascii_uppercase())
            });

            Ok(KunosServer {
                ip: entry.ip,
                http_port: entry.cport,
                udp_port: entry.port.filter(|port| *port != 0),
                game_port: entry.tport.filter(|port| *port != 0),
                name: entry.name,
                track: entry.track,
                players: entry.clients,
                max_players: entry.maxclients,
                cars: entry.cars,
                country,
                password_required: entry.pass,
            })
        })
        .collect()
}

#[tauri::command]
pub(crate) async fn get_kunos_online_servers(steam_id: String) -> Result<Vec<KunosServer>, String> {
    if steam_id.len() != 17 || !steam_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Un Steam ID 64 valide est nécessaire pour accéder au lobby Kunos.".into());
    }

    let client = reqwest::Client::builder()
        .user_agent("Assetto Corsa Launcher")
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("Impossible de créer le client Kunos : {error}"))?;

    fetch_lobby(&client, LOBBY_URL, &steam_id).await
}

async fn fetch_lobby(client: &reqwest::Client, url: &str, steam_id: &str) -> Result<Vec<KunosServer>, String> {
    let mut response = client
        .get(url)
        .query(&[("guid", steam_id)])
        .send()
        .await
        .map_err(|error| format!("Connexion au lobby Kunos impossible : {}", error.without_url()))?;

    if !response.status().is_success() {
        return Err(format!("Le lobby Kunos a répondu avec le statut HTTP {}.", response.status()));
    }
    if response.content_length().is_some_and(|size| size > MAX_RESPONSE_BYTES as u64) {
        return Err("La réponse du lobby Kunos est trop volumineuse.".into());
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Lecture du lobby Kunos impossible : {}", error.without_url()))?
    {
        if chunk.len() > MAX_RESPONSE_BYTES - body.len() {
            return Err("La réponse du lobby Kunos est trop volumineuse.".into());
        }
        body.extend_from_slice(&chunk);
    }
    parse_lobby_response(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kunos_listing_and_distinguishes_ports() {
        let body = br#"[{
            "ip":"192.0.2.1","cport":8081,"port":9600,"tport":9601,
            "name":"Test","track":"monza","clients":3,"maxclients":24,
            "cars":["car_a","car_b"],"country":["France","fr"],"pass":true
        }]"#;

        let servers = parse_lobby_response(body).unwrap();
        let server = &servers[0];
        assert_eq!((server.http_port, server.udp_port, server.game_port), (8081, Some(9600), Some(9601)));
        assert_eq!(server.country.as_deref(), Some("FR"));
        assert_eq!(server.players, Some(3));
        assert_eq!(server.cars.as_ref().unwrap().len(), 2);
        assert_eq!(server.password_required, Some(true));
    }

    #[test]
    fn handles_optional_fields_and_rejects_bad_responses() {
        let servers = parse_lobby_response(br#"[{"ip":"192.0.2.1","cport":8081,"country":null}]"#).unwrap();
        assert!(servers[0].country.is_none());
        assert!(servers[0].game_port.is_none());
        assert!(parse_lobby_response(b"ERROR,SLOW DOWN,TOO MANY CONCURRENT REQUESTS")
            .unwrap_err().contains("limite les requêtes"));
        assert!(parse_lobby_response(br#"[{"ip":"192.0.2.1","cport":0}]"#).is_err());
        assert!(parse_lobby_response(br#"[{"ip":"not-an-ip","cport":8081}]"#).is_err());
        assert!(parse_lobby_response(br#"[{"ip":"192.168.1.1","cport":8081},{"ip":"192.168.1.1","cport":8082}]"#)
            .unwrap_err().contains("Steam ID"));
    }

    #[tokio::test]
    async fn rejects_invalid_steam_id_before_network_access() {
        assert!(get_kunos_online_servers("invalid".into()).await.unwrap_err().contains("Steam ID"));
    }

    #[tokio::test]
    async fn fetches_mock_lobby_response() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 512];
            while !request.ends_with(b"\r\n\r\n") && request.len() < 2048 {
                let count = socket.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
            }
            let request = std::str::from_utf8(&request).unwrap();
            assert!(request.starts_with("GET /lobby.ashx/list?guid=76561197960265728 "));
            let body = r#"[{"ip":"192.0.2.1","cport":8081,"tport":9600}]"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build().unwrap();
        let url = format!("http://{address}/lobby.ashx/list");
        let servers = fetch_lobby(&client, &url, "76561197960265728").await.unwrap();
        server.await.unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].http_port, 8081);
        assert_eq!(servers[0].game_port, Some(9600));
    }
}
