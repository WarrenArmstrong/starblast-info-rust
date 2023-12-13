use futures_util::{stream::StreamExt, SinkExt};
use rand::seq::SliceRandom;
use rand::Rng;
use serde_json::json;
use tokio_tungstenite::{
    connect_async, tungstenite::client::IntoClientRequest, tungstenite::protocol::Message,
};

pub async fn listen_system(server_address: String, system_id: i32) {
    // let mut rng = rand::thread_rng();

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
                    if let Some(200) = binary.get(0) {
                        // write.send(Message::Text("100".into())).await.unwrap();
                        write
                            .send(Message::Text(
                                rand::Rng::gen_range(&mut rand::rngs::OsRng, 0..=359).to_string(),
                            ))
                            .await
                            .unwrap();
                    }
                    // println!("{} {:?}", system_id, binary);
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
