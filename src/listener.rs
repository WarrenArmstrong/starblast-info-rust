use crate::system_listener::SystemListener;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::{task::JoinHandle, time};

pub async fn listen_simstatus() {
    let mut interval = time::interval(time::Duration::from_secs(10));

    let mut system_listeners: HashMap<i32, SystemListener> = HashMap::new();

    loop {
        interval.tick().await;

        let new_sim_status: Vec<ServerDTO> = reqwest::Client::new()
            .get("https://starblast.io/simstatus.json")
            .send()
            .await
            .expect("simstatus failed to send")
            .json()
            .await
            .expect("simstatus failed to parse");

        println!("parsed simstatus");

        // remove stale system listeners
        system_listeners.retain(|system_id, system_listener| {
            let task_finished = system_listener.handle.is_finished();
            if task_finished {
                println!("{} deleted system listener", system_id);
            }
            !task_finished
        });

        // create new system listeners
        new_sim_status.iter().for_each(|server| {
            server
                .systems
                .iter()
                .filter(|system| system.mode == "team")
                .for_each(|system| {
                    let system_listeners_ref = &mut system_listeners;
                    if !system_listeners_ref.contains_key(&system.id) {
                        let system_listener =
                            SystemListener::new(server.address.clone(), system.id);

                        system_listeners_ref.insert(system.id, system_listener);
                        println!("{} started system listener", system.id);
                    }
                })
        });

        println!("{:?}", system_listeners.keys())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SystemDTO {
    name: String,
    id: i32,
    mode: String,
    players: i32,
    unlisted: bool,
    open: bool,
    survival: bool,
    time: i32,
    criminal_activity: i32,
    mod_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UsageDTO {
    cpu: i32,
    memory: i32,
    ctime: Option<i32>,
    elapsed: Option<f64>,
    timestamp: Option<i64>,
    pid: Option<i32>,
    ppid: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerDTO {
    location: String,
    address: String,
    current_players: i32,
    systems: Vec<SystemDTO>,
    usage: UsageDTO,
}
