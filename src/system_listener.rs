use futures_util::{stream::StreamExt, SinkExt};
use rand::seq::SliceRandom;
use rand::Rng;
use serde_json::json;
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::protocol::Message,
};

use std::{fmt::Binary, io::Error};

pub async fn listen_system(server_address: String, system_id: i32) {
    // establish ws streams

    let (ip, port) = server_address.split_once(':').unwrap();
    let dashed_ip = ip.replace(".", "-");
    let websocket_endpoint = format!("wss://{dashed_ip}.starblast.io:{port}/");

    let mut request = websocket_endpoint.into_client_request().unwrap();
    let headers = request.headers_mut();
    headers.insert("Origin", "https://starblast.io".parse().unwrap());

    let (ws_stream, _) = connect_async(request)
        .await
        .expect(&format!("{} failed to connect", system_id));

    println!(
        "{} websocket handshake has been successfully completed",
        system_id
    );

    let (mut write, mut read) = ws_stream.split();

    write
        .send(Message::Text(get_initial_message(system_id)))
        .await
        .unwrap();

    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(message) => match message {
                Message::Text(text) => {
                    // println!("{} {:?}", system_id, text);
                }
                Message::Binary(binary) => {
                    let binary_dto_result = BinaryDTO::parse(&binary);

                    if let Ok(BinaryDTO::Binary200DTO(_)) = binary_dto_result {
                        write
                            .send(Message::Text(
                                rand::Rng::gen_range(&mut rand::rngs::OsRng, 0..=359).to_string(),
                            ))
                            .await
                            .unwrap();
                    }

                    match BinaryDTO::parse(&binary) {
                        Ok(binary_dto) => match binary_dto {
                            BinaryDTO::Binary200DTO(binary_200_dto) => {}
                            BinaryDTO::Binary205DTO(binary_205_dto) => {}
                            BinaryDTO::Binary206DTO(binary_206_dto) => {}
                        },
                        Err(e) => {
                            println!("{} error when parsing binary: {:?}", system_id, binary);
                        }
                    }
                }
                Message::Ping(_ping) => {
                    println!("{} recieved ping", system_id)
                }
                Message::Pong(_pong) => {
                    println!("{} recieved ping", system_id)
                }
                Message::Close(_close_frame_option) => {
                    println!("{} recieved close", system_id)
                }
            },
            Err(e) => {
                println!("{} Error reading message: {}", system_id, e);
            }
        }
    }

    println!("{} reached end of listener", system_id);
}

// write
// .send(Message::text(
//     serde_json::to_string(
//         &json!({"name": "get_name", "data": {"id": player_sb_id}}),
//     )
//     .unwrap(),
// ))
// .await
// .unwrap();

fn get_initial_message(system_id: i32) -> String {
    let mut rng = rand::thread_rng();

    let client_ship_id: String = rng
        .gen_range(100000000000000000 as i64..999999999999999999 as i64)
        .to_string();

    let default_names = vec![
        "Arkady Darell",
        "Bel Riose",
        "Cleon I",
        "Dors Venabili",
        "Ebling Mis",
        "Gaal Dornick",
        "Hari Seldon",
        "Hober Mallow",
        "Janov Pelorat",
        "The Mule",
        "Preem Palver",
        "R.D. Olivaw",
        "R.G. Reventlov",
        "Raych Seldon",
        "Salvor Hardin",
        "Wanda Seldon",
        "Yugo Amaryl",
        "James T. Kirk",
        "Leonard McCoy",
        "Hikaru Sulu",
        "Montgomery Scott",
        "Spock",
        "Picard",
        "Christine Chapel",
        "Nyota Uhura",
        "Pavel Chekov",
        "Ford",
        "Zaphod",
        "Marvin",
        "Anakin",
        "Luke",
        "Leia",
        "Ackbar",
        "Tarkin",
        "Jabba",
        "Rey",
        "Kylo",
        "Han",
        "Vader",
        "D.A.R.Y.L.",
        "HAL 9000",
        "Lyta Alexander",
        "Stephen Franklin",
        "Lennier",
    ];

    let player_name = default_names.choose(&mut rng).unwrap().to_uppercase();

    let initial_message = json!({
        "name": "ùov()",
        "data": {
            "mode": "join",
            "player_name": player_name,
            "hue": 12,
            "preferred": system_id,
            "bonus": false,
            "ecp_key": null,
            "steamid": null,
            "ecp_custom": {
                "badge": "star",
                "finish": "alloy",
                "laser": "1",
            },
            "create": false,
            "client_ship_id": client_ship_id,
            "client_tr": 1,
        },
    });

    serde_json::to_string(&initial_message).unwrap()
}

#[derive(Debug)]
struct Binary200PlayerDTO {
    id: u8,
    score: u32,
    ship_id: u32,
    is_alive: bool,
    x: u8,
    y: u8,
}

impl Binary200PlayerDTO {
    fn parse(binary: &[u8; 8]) -> Result<Self, Error> {
        let id = binary[0];
        let score = 0xFFFFFF & u32::from_le_bytes(binary[4..8].try_into().unwrap());

        let ship_index = 1 + (u32::from_le_bytes(binary[4..8].try_into().unwrap()) >> 24);
        let ship_level = 1 + ((binary[3] >> 5) & 7);
        let ship_id = (100 * u32::from(ship_level)) + ship_index;

        let is_alive = (1 & binary[3]) == 1;
        let x = binary[1];
        let y = binary[2];

        Ok(Self {
            id,
            score,
            ship_id,
            is_alive,
            x,
            y,
        })
    }
}

#[derive(Debug)]
struct Binary200DTO {
    msg_type: u8,
    player_count: u8,
    players: Vec<Binary200PlayerDTO>,
}

impl Binary200DTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        let msg_type = binary[0];
        let player_count = binary[1];

        let content_bytes = &binary[2..];
        let mut players = Vec::new();

        for chunk in content_bytes.chunks_exact(8).take(player_count as usize) {
            let player = Binary200PlayerDTO::parse(chunk.try_into().unwrap()).unwrap();
            players.push(player);
        }

        Ok(Self {
            msg_type,
            player_count,
            players,
        })
    }
}

#[derive(Debug, Clone)]
struct Binary205ModuleDTO {
    id: u8,
    status: u8,
}

#[derive(Debug, Clone)]
struct Binary205TeamDTO {
    id: u8,
    is_open: bool,
    base_level: u8,
    crystals: u32,
    module_count: u8,
    modules: Vec<Binary205ModuleDTO>,
}

impl Binary205TeamDTO {
    fn parse(binary: &[u8], idx: u8) -> Result<Self, Error> {
        let is_open = binary[0] == 1;
        let base_level = binary[1];
        let crystals = u32::from_le_bytes(binary[2..6].try_into().unwrap());
        let module_count = binary[6];

        let modules_bytes = &binary[7..];
        let modules = modules_bytes
            .iter()
            .enumerate()
            .map(|(idx, &status)| Binary205ModuleDTO {
                id: idx as u8,
                status: status,
            })
            .collect();

        Ok(Self {
            id: idx,
            is_open,
            base_level,
            crystals,
            module_count,
            modules,
        })
    }
}

#[derive(Debug, Clone)]
struct Binary205DTO {
    msg_type: u8,
    teams: Vec<Binary205TeamDTO>,
}

impl Binary205DTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        let msg_type = binary[0];
        let content_bytes = &binary[1..];

        let teams = (0..3)
            .map(|i| {
                let team_bytes = &content_bytes[19 * i..19 * (i + 1)];
                Binary205TeamDTO::parse(team_bytes, i as u8).unwrap()
            })
            .collect();

        Ok(Self { msg_type, teams })
    }
}

#[derive(Debug, Clone)]
struct Binary206AlienDTO {
    id: u16,
    x: u8,
    y: u8,
}

impl Binary206AlienDTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        let id = u16::from_le_bytes(binary[0..2].try_into().unwrap());
        let x = binary[3];
        let y = binary[4];

        Ok(Self { id, x, y })
    }
}

#[derive(Debug, Clone)]
struct Binary206DTO {
    msg_type: u8,
    wave: u8,
    wave_start_time: u32,
    alien_count: u16,
    aliens: Vec<Binary206AlienDTO>,
}

impl Binary206DTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        let msg_type = binary[0];
        let wave = binary[1];
        let wave_start_time = u32::from_le_bytes(binary[2..6].try_into().unwrap());
        let alien_count = u16::from_le_bytes(binary[8..10].try_into().unwrap());

        let aliens = (0..alien_count as usize)
            .map(|i| {
                let alien_bytes = &binary[10 + (5 * i)..10 + (5 * (i + 1))];
                Binary206AlienDTO::parse(alien_bytes).unwrap()
            })
            .collect();

        Ok(Self {
            msg_type,
            wave,
            wave_start_time,
            alien_count,
            aliens,
        })
    }
}

enum BinaryDTO {
    Binary200DTO(Binary200DTO),
    Binary205DTO(Binary205DTO),
    Binary206DTO(Binary206DTO),
}

impl BinaryDTO {
    fn parse(binary: &[u8]) -> Result<Self, Error> {
        Ok(match binary[0] {
            200 => BinaryDTO::Binary200DTO(Binary200DTO::parse(binary)?),
            205 => BinaryDTO::Binary205DTO(Binary205DTO::parse(binary)?),
            206 => BinaryDTO::Binary206DTO(Binary206DTO::parse(binary)?),
            _ => todo!(),
        })
    }
}

struct SystemListenerData {
    binary_200_dto: Option<Binary200DTO>,
    binary_205_dto: Option<Binary205DTO>,
    binary_206_dto: Option<Binary206DTO>,
}
