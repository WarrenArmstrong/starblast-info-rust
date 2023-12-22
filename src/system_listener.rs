use futures_util::{stream::StreamExt, SinkExt};
use rand::seq::SliceRandom;
use rand::Rng;
use serde_json::json;
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::protocol::Message,
};

use std::{fmt::Binary, io::Error};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use std::future::Future;

async fn listen_system(
    server_address: String,
    system_id: i32,
    data: Arc<Mutex<SystemListenerData>>,
) {
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

    // println!(
    //     "{} websocket handshake has been successfully completed",
    //     system_id
    // );

    let (mut write, mut read) = ws_stream.split();

    write
        .send(Message::Text(get_initial_message(system_id)))
        .await
        .unwrap();

    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(message) => match message {
                Message::Text(text) => {
                    // println!("{} {}", system_id, text);

                    let ws_dto_result: Result<WSDTO, serde_json::Error> =
                        serde_json::from_str(&text);

                    match ws_dto_result {
                        Ok(ws_dto) => match ws_dto {
                            WSDTO::WSWelcomeDTO(ws_welcome_dto) => {
                                data.lock().unwrap().ws_welcome_dto = Some(ws_welcome_dto);
                                println!("{} parsed WSWelcome", system_id)
                            }
                            WSDTO::WSPlayerNameDTO(ws_player_name_dto) => {
                                data.lock()
                                    .unwrap()
                                    .ws_player_name_dtos
                                    .insert(ws_player_name_dto.data.id, ws_player_name_dto);
                            }
                            WSDTO::WSCannotJoin => {
                                write.send(Message::Close(None)).await.unwrap();
                                break;
                            }
                        },
                        Err(e) => {
                            println!("{} {} {}", system_id, e, text);
                        }
                    }
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

                        let data_gaurd = data.lock().unwrap();

                        for (k, v) in data_gaurd.ws_player_name_dtos.iter() {
                            if v.data.player_name == "TEST123" {
                                for player in
                                    data_gaurd.binary_200_dto.as_ref().unwrap().players.iter()
                                {
                                    if player.id == v.data.id {
                                        println!("{:?} {:?}", v.data, player);
                                    }
                                }
                            }
                        }
                    }

                    match BinaryDTO::parse(&binary) {
                        Ok(binary_dto) => match binary_dto {
                            BinaryDTO::Binary200DTO(binary_200_dto) => {
                                let binary_player_ids: HashSet<u8> = HashSet::from_iter(
                                    binary_200_dto.players.iter().map(|player| player.id),
                                );

                                // let data_gaurd = data.lock().unwrap();

                                data.lock()
                                    .unwrap()
                                    .ws_player_name_dtos
                                    .retain(|id, _| binary_player_ids.contains(id));

                                // let futures: Vec<Box<dyn Future>> = binary_200_dto
                                //     .players
                                //     .iter()
                                //     .filter(|player| {
                                //         !data_gaurd.ws_player_name_dtos.contains_key(&player.id)
                                //     })
                                //     .map(|player| write.send(Message::Text("test".into())))
                                //     .collect();

                                let current_player_ids: Vec<u8> = data
                                    .lock()
                                    .unwrap()
                                    .ws_player_name_dtos
                                    .keys()
                                    .cloned()
                                    .collect();

                                for player in binary_200_dto.players.iter() {
                                    if !current_player_ids.contains(&player.id) {
                                        write
                                        .send(Message::text(
                                            serde_json::to_string(
                                                &json!({"name": "get_name", "data": {"id": player.id}}),
                                            )
                                            .unwrap(),
                                        ))
                                        .await
                                        .unwrap();
                                    }
                                }

                                data.lock().unwrap().binary_200_dto = Some(binary_200_dto);
                            }
                            BinaryDTO::Binary205DTO(binary_205_dto) => {
                                data.lock().unwrap().binary_205_dto = Some(binary_205_dto);
                            }
                            BinaryDTO::Binary206DTO(binary_206_dto) => {
                                data.lock().unwrap().binary_206_dto = Some(binary_206_dto);
                            }
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

fn get_initial_message(system_id: i32) -> String {
    let mut rng = rand::thread_rng();

    let client_ship_id: String = rng
        .gen_range(100000000000000000 as i64..999999999999999999 as i64)
        .to_string();

    let default_names = vec![
        "ARKADY DARELL",
        "BEL RIOSE",
        "CLEON I",
        "DORS VENABILI",
        "EBLING MIS",
        "GAAL DORNICK",
        "HARI SELDON",
        "HOBER MALLOW",
        "JANOV PELORAT",
        "THE MULE",
        "PREEM PALVER",
        "R.D. OLIVAW",
        "R.G. REVENTLOV",
        "RAYCH SELDON",
        "SALVOR HARDIN",
        "WANDA SELDON",
        "YUGO AMARYL",
        "JAMES T. KIRK",
        "LEONARD MCCOY",
        "HIKARU SULU",
        "MONTGOMERY SCOTT",
        "SPOCK",
        "PICARD",
        "CHRISTINE CHAPEL",
        "NYOTA UHURA",
        "PAVEL CHEKOV",
        "FORD",
        "ZAPHOD",
        "MARVIN",
        "ANAKIN",
        "LUKE",
        "LEIA",
        "ACKBAR",
        "TARKIN",
        "JABBA",
        "REY",
        "KYLO",
        "HAN",
        "VADER",
        "D.A.R.Y.L.",
        "HAL 9000",
        "LYTA ALEXANDER",
        "STEPHEN FRANKLIN",
        "LENNIER",
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
        Ok(match binary.get(0) {
            Some(200) => BinaryDTO::Binary200DTO(Binary200DTO::parse(binary)?),
            Some(205) => BinaryDTO::Binary205DTO(Binary205DTO::parse(binary)?),
            Some(206) => BinaryDTO::Binary206DTO(Binary206DTO::parse(binary)?),
            _ => todo!(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSPlayerNameCustomDTO {
    badge: String,
    finish: String,
    laser: Option<String>,
    hue: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSPlayerNameDataDTO {
    id: u8,
    hue: u16,
    friendly: u8,
    player_name: String,
    custom: Option<WSPlayerNameCustomDTO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSPlayerNameDTO {
    name: String,
    data: WSPlayerNameDataDTO,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSWelcomeModuleDTO {
    id: u8,
    #[serde(rename = "type")]
    module_type: String,
    x: f64,
    y: f64,
    dir: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSWelcomeStationDTO {
    modules: Vec<WSWelcomeModuleDTO>,
    phase: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSWelcomeTeamDTO {
    faction: String,
    base_name: String,
    hue: u16,
    station: WSWelcomeStationDTO,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSWelcomeModeDTO {
    max_players: i32,
    crystal_value: f64,
    crystal_drop: i32,
    map_size: i32,
    map_density: Option<i32>,
    lives: i32,
    max_level: i32,
    friendly_colors: i32,
    close_time: i32,
    close_number: i32,
    map_name: Option<String>,
    unlisted: bool,
    survival_time: i32,
    survival_level: i32,
    starting_ship: i32,
    starting_ship_maxed: bool,
    asteroids_strength: i32,
    friction_ratio: f64,
    speed_mod: f64,
    rcs_toggle: bool,
    weapon_drop: i32,
    mines_self_destroy: bool,
    mines_destroy_delay: i32,
    healing_enabled: bool,
    healing_ratio: f64,
    shield_regen_factor: f64,
    power_regen_factor: f64,
    auto_refill: bool,
    projectile_speed: f64,
    strafe: f64,
    release_crystal: bool,
    large_grid: bool,
    bouncing_lasers: i32,
    max_tier_lives: i32,
    auto_assign_teams: bool,
    station_size: i32,
    crystal_capacity: Vec<i32>,
    deposit_shield: Vec<i32>,
    spawning_shield: Vec<i32>,
    structure_shield: Vec<i32>,
    deposit_regen: Vec<i32>,
    spawning_regen: Vec<i32>,
    structure_regen: Vec<i32>,
    repair_threshold: f64,
    all_ships_can_dock: bool,
    all_ships_can_respawn: bool,
    id: String,
    restore_ship: Option<bool>,
    teams: Vec<WSWelcomeTeamDTO>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSWelcomeDataDTO {
    version: i32,
    seed: i32,
    servertime: i32,
    name: String,
    systemid: i32,
    size: i32,
    mode: WSWelcomeModeDTO,
    region: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WSWelcomeDTO {
    name: String,
    data: WSWelcomeDataDTO,
}

#[derive(Debug, Serialize, Clone)]
enum WSDTO {
    WSWelcomeDTO(WSWelcomeDTO),
    WSPlayerNameDTO(WSPlayerNameDTO),
    WSCannotJoin,
}

use serde::{de, Deserializer};
use serde_json::Value;

impl<'de> Deserialize<'de> for WSDTO {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        match v.get("name").and_then(Value::as_str) {
            Some("player_name") => {
                let data = WSPlayerNameDTO::deserialize(v).map_err(de::Error::custom)?;
                Ok(WSDTO::WSPlayerNameDTO(data))
            }
            Some("welcome") => {
                let data = WSWelcomeDTO::deserialize(v).map_err(de::Error::custom)?;
                Ok(WSDTO::WSWelcomeDTO(data))
            }
            Some("cannot_join") => Ok(WSDTO::WSCannotJoin),
            _ => Err(serde::de::Error::unknown_variant(
                v.get("name").unwrap().as_str().unwrap(),
                &["player_name", "welcome", "cannot_join"],
            )),
        }
    }
}

// struct WSDTO {
//     name: String,
//     data: Option<
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// enum WSDTOName {
//     #[serde(rename = "welcome")]
//     Welcome,
//     #[serde(rename = "player_name")]
//     PlayerName,
//     #[serde(rename = "cannot_join")]
//     CannotJoin,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct WSDTO {
//     name: String,
//     data: Option<WSDTOData>,
// }

// impl WSDTO {
//     fn parse(text: String) -> Result<Self, Error> {
//         Ok(match binary.get(0) {
//             Some(200) => BinaryDTO::Binary200DTO(Binary200DTO::parse(binary)?),
//             Some(205) => BinaryDTO::Binary205DTO(Binary205DTO::parse(binary)?),
//             Some(206) => BinaryDTO::Binary206DTO(Binary206DTO::parse(binary)?),
//             _ => todo!(),
//         })
//     }
// }

struct SystemListenerData {
    ws_welcome_dto: Option<WSWelcomeDTO>,
    ws_player_name_dtos: HashMap<u8, WSPlayerNameDTO>,
    binary_200_dto: Option<Binary200DTO>,
    binary_205_dto: Option<Binary205DTO>,
    binary_206_dto: Option<Binary206DTO>,
}

use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

pub struct SystemListener {
    pub data: Arc<Mutex<SystemListenerData>>,
    pub handle: JoinHandle<()>,
}

impl SystemListener {
    pub fn new(server_address: String, system_id: i32) -> Self {
        let data = Arc::new(Mutex::new(SystemListenerData {
            ws_welcome_dto: None,
            ws_player_name_dtos: HashMap::new(),
            binary_200_dto: None,
            binary_205_dto: None,
            binary_206_dto: None,
        }));

        let handle = tokio::spawn(listen_system(server_address, system_id, Arc::clone(&data)));

        Self {
            data: data,
            handle: handle,
        }
    }
}
