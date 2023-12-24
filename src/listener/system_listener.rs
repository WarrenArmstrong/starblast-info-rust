use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::protocol::Message,
    tungstenite::Error,
};

use futures_util::{stream::StreamExt, SinkExt};

use std::collections::{HashMap, HashSet};

use rand::Rng;

use serde_json::json;
use serde_json::Error as SJError;

use super::dto::{Binary200DTO, Binary205DTO, Binary206DTO, WSPlayerNameDTO, WSWelcomeDTO, WSDTO};

pub struct SystemListener {
    system_id: i32,
    server_address: String,
    websocket_endpoint: String,

    ws_welcome_dto: Option<WSWelcomeDTO>,
    ws_player_name_dtos: HashMap<u8, WSPlayerNameDTO>,

    binary_200_dto: Option<Binary200DTO>,
    binary_205_dto: Option<Binary205DTO>,
    binary_206_dto: Option<Binary206DTO>,
}

impl SystemListener {
    pub fn new(system_id: i32, server_address: String) -> Self {
        let (ip, port) = server_address.split_once(':').unwrap();
        let dashed_ip = ip.replace(".", "-");
        let websocket_endpoint = format!("wss://{dashed_ip}.starblast.io:{port}/");

        Self {
            system_id: system_id,
            server_address: server_address,
            websocket_endpoint: websocket_endpoint,

            ws_welcome_dto: None,
            ws_player_name_dtos: HashMap::new(),

            binary_200_dto: None,
            binary_205_dto: None,
            binary_206_dto: None,
        }
    }

    pub async fn listen(self) {
        if let Err(e) = self.listen_process().await {
            println!("{}", e);
        }
    }

    async fn listen_process(mut self) -> Result<(), Error> {
        let mut request = self.websocket_endpoint.clone().into_client_request()?;
        let headers = request.headers_mut();
        headers.insert("Origin", "https://starblast.io".parse()?);

        let (ws_stream, _) = connect_async(request).await?;

        let (mut write, mut read) = ws_stream.split();

        write
            .send(Message::Text(
                self.get_initial_message().map_err(|e| Error::Utf8)?,
            ))
            .await?;

        while let Some(message_result) = read.next().await {
            let message = match message_result {
                Ok(message) => message,
                Err(e) => {
                    println!("{} error reading message: {}", self.system_id, e);
                    continue;
                }
            };

            let ws_dto = match WSDTO::parse(message.clone()) {
                Ok(ws_dto) => ws_dto,
                Err(e) => {
                    println!(
                        "{} error parsing WSDTO error: {} message: {}",
                        self.system_id, e, message
                    );
                    continue;
                }
            };

            if let WSDTO::Binary200DTO(_) = ws_dto {
                write.send(Message::Text("100".into())).await?
            }

            match ws_dto {
                WSDTO::WSWelcomeDTO(ws_welcome_dto) => {
                    self.ws_welcome_dto = Some(ws_welcome_dto);
                    println!("{} parsed WSWelcome", self.system_id)
                }
                WSDTO::WSPlayerNameDTO(ws_player_name_dto) => {
                    println!("{} {:?}", self.system_id, ws_player_name_dto.data);

                    self.ws_player_name_dtos
                        .insert(ws_player_name_dto.data.id, ws_player_name_dto);
                }
                WSDTO::CannotJoin => {
                    write.send(Message::Close(None)).await?;
                    break;
                }
                WSDTO::Binary200DTO(binary_200_dto) => {
                    let binary_player_ids: HashSet<u8> =
                        HashSet::from_iter(binary_200_dto.players.iter().map(|p| p.id));

                    self.ws_player_name_dtos
                        .retain(|id, _| binary_player_ids.contains(id));

                    let current_player_ids: HashSet<u8> =
                        self.ws_player_name_dtos.keys().copied().collect();

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

                    self.binary_200_dto = Some(binary_200_dto);
                }
                WSDTO::Binary205DTO(binary_205_dto) => self.binary_205_dto = Some(binary_205_dto),
                WSDTO::Binary206DTO(binary_206_dto) => self.binary_206_dto = Some(binary_206_dto),
                WSDTO::Ping(ping) => {
                    println!("{} recieved ping: {:?}", self.system_id, ping)
                }
                WSDTO::Pong(pong) => {
                    println!("{} recieved pong: {:?}", self.system_id, pong)
                }
                WSDTO::Close(close_frame_option) => {
                    println!(
                        "{} recieved close frame: {:?}",
                        self.system_id, close_frame_option
                    )
                }
            }
        }

        Ok(())
    }

    fn get_initial_message(&self) -> Result<String, SJError> {
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
                "preferred": self.system_id,
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
}

static NAMES: &[&'static str] = &[
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
