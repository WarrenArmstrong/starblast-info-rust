use futures_util::{stream::StreamExt, SinkExt};
use rand::Rng;
use serde_json::{json, Error as SJError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::{
    sync::{
        watch::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::protocol::Message,
    tungstenite::Error,
};

use crate::utils::NAMES;

use super::dto::{Binary200DTO, Binary205DTO, Binary206DTO, WSPlayerNameDTO, WSWelcomeDTO, WSDTO};

pub struct SystemListener {
    pub handle: JoinHandle<()>,
    pub rx: Receiver<Option<Message>>,
}

struct SystemListenerData {
    system_id: i32,
    server_address: String,

    tx: Sender<Option<Message>>,

    ws_welcome_dto: Option<Arc<WSWelcomeDTO>>,
    ws_player_name_dtos: HashMap<u8, Arc<WSPlayerNameDTO>>,

    binary_200_dto: Option<Binary200DTO>,
    binary_205_dto: Option<Binary205DTO>,
    binary_206_dto: Option<Binary206DTO>,
}

pub fn listen_system(system_id: i32, server_address: String) -> SystemListener {
    let (tx, rx) = watch::channel(None);

    let data = SystemListenerData {
        system_id: system_id,
        server_address: server_address,

        tx: tx,

        ws_welcome_dto: None,
        ws_player_name_dtos: HashMap::new(),

        binary_200_dto: None,
        binary_205_dto: None,
        binary_206_dto: None,
    };

    let handle = tokio::spawn(listen_system_error_handle(data));

    SystemListener {
        handle: handle,
        rx: rx,
    }
}

async fn listen_system_error_handle(data: SystemListenerData) {
    let system_id = data.system_id.clone();

    if let Err(e) = listen_system_process(data).await {
        println!("{} error while listening: {}", system_id, e);
    }
}

async fn listen_system_process(mut data: SystemListenerData) -> Result<(), Error> {
    let (ip, port) = data.server_address.split_once(':').unwrap();
    let dashed_ip = ip.replace(".", "-");
    let websocket_endpoint = format!("wss://{dashed_ip}.starblast.io:{port}/");

    let mut request = websocket_endpoint.into_client_request()?;
    let headers = request.headers_mut();
    headers.insert("Origin", "https://starblast.io".parse()?);

    let (ws_stream, _) = connect_async(request).await?;

    let (mut write, mut read) = ws_stream.split();

    let initial_message = Message::Text(get_initial_message(data.system_id).unwrap());
    write.send(initial_message).await?;

    while let Some(message_result) = read.next().await {
        let message = match message_result {
            Ok(message) => message,
            Err(e) => {
                println!("{} error reading message: {}", data.system_id, e);
                continue;
            }
        };

        match data.tx.send(Some(message.clone())) {
            Err(e) => println!("{:?}", e),
            Ok(()) => {}
        };

        let ws_dto = match WSDTO::parse(message.clone()) {
            Ok(ws_dto) => ws_dto,
            Err(e) => {
                println!(
                    "{} error parsing WSDTO error: {} message: {}",
                    data.system_id, e, message
                );
                continue;
            }
        };

        if let WSDTO::Binary200DTO(_) = ws_dto {
            let text = rand::Rng::gen_range(&mut rand::rngs::OsRng, 0..=359).to_string();
            write.send(Message::Text(text)).await?;
        }

        match ws_dto {
            WSDTO::WSWelcomeDTO(ws_welcome_dto) => {
                data.ws_welcome_dto = Some(Arc::new(ws_welcome_dto));
                println!("{} parsed WSWelcome", data.system_id)
            }
            WSDTO::WSPlayerNameDTO(ws_player_name_dto) => {
                // println!("{} {:?}", self.system_id, ws_player_name_dto.data);

                data.ws_player_name_dtos
                    .insert(ws_player_name_dto.data.id, Arc::new(ws_player_name_dto));
            }
            WSDTO::CannotJoin => {
                println!("{} recieved cannot_join", data.system_id);
                write.send(Message::Close(None)).await?;
                break;
            }
            WSDTO::Binary200DTO(binary_200_dto) => {
                let binary_player_ids: HashSet<u8> =
                    HashSet::from_iter(binary_200_dto.players.iter().map(|p| p.id));

                data.ws_player_name_dtos
                    .retain(|id, _| binary_player_ids.contains(id));

                let current_player_ids: HashSet<u8> =
                    data.ws_player_name_dtos.keys().copied().collect();

                let messages: Vec<Message> = binary_200_dto
                    .players
                    .iter()
                    .filter(|p| !current_player_ids.contains(&p.id))
                    .map(|p| json!({"name": "get_name", "data": {"id": p.id}}))
                    .map(|j| serde_json::to_string(&j).unwrap())
                    .map(|s| Message::Text(s))
                    .collect();

                for message in messages {
                    write.send(message).await?
                }

                data.binary_200_dto = Some(binary_200_dto);
            }
            WSDTO::Binary205DTO(binary_205_dto) => data.binary_205_dto = Some(binary_205_dto),
            WSDTO::Binary206DTO(binary_206_dto) => data.binary_206_dto = Some(binary_206_dto),
            WSDTO::Ping(ping) => {
                println!("{} recieved ping: {:?}", data.system_id, ping)
            }
            WSDTO::Pong(pong) => {
                println!("{} recieved pong: {:?}", data.system_id, pong)
            }
            WSDTO::Close(close_frame_option) => {
                println!(
                    "{} recieved close frame: {:?}",
                    data.system_id, close_frame_option
                )
            }
        }
    }

    Ok(())
}

fn get_initial_message(system_id: i32) -> Result<String, SJError> {
    let mut rng = rand::thread_rng();

    let client_ship_id: String = rng
        .gen_range(100000000000000000 as i64..999999999999999999 as i64)
        .to_string();

    let player_name = NAMES[rng.gen_range(0..NAMES.len())];

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

    serde_json::to_string(&initial_message)
}
