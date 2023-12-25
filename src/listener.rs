mod dto;
mod system_listener;

use dto::{ServerDTO, SimStatusDTO};
use reqwest::Error;
use std::collections::{HashMap, HashSet};
use std::{alloc::System, sync::Arc};
use system_listener::listen_system;
use tokio::{
    sync::watch::{self, Receiver, Sender},
    task::JoinHandle,
    time,
};

use self::system_listener::SystemListener;

pub struct Listener {
    pub handle: JoinHandle<()>,
    pub rx: Receiver<HashSet<i32>>,
}

struct ListenerData {
    system_listeners: HashMap<i32, SystemListener>,
    tx: Sender<HashSet<i32>>,
}

pub fn listen() -> Listener {
    let (tx, rx) = watch::channel(HashSet::new());

    let data = ListenerData {
        system_listeners: HashMap::new(),
        tx: tx,
    };

    let handle = tokio::spawn(listen_process(data));

    Listener {
        handle: handle,
        rx: rx,
    }
}

async fn listen_process(mut data: ListenerData) {
    let mut interval = time::interval(time::Duration::from_secs(10));

    loop {
        interval.tick().await;

        let response = match reqwest::Client::new()
            .get("https://starblast.io/simstatus.json")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("Error sending request: {:?}", e);
                continue;
            }
        };

        let simstatus = match response.json::<SimStatusDTO>().await {
            Ok(simstatus) => simstatus,
            Err(e) => {
                println!("Error parsing response: {:?}", e);
                continue;
            }
        };

        println!("parsed simstatus");

        data.system_listeners.retain(|k, v| {
            if v.handle.is_finished() {
                println!("{} deleted system listener", k);
            };
            !v.handle.is_finished()
        });

        for server in simstatus.servers {
            for system in server.systems {
                if system.mode == "team" && !data.system_listeners.contains_key(&system.id) {
                    let system_listener = listen_system(system.id, server.address.clone());

                    data.system_listeners.insert(system.id, system_listener);

                    println!("{} started system listener", system.id);
                }
            }
        }

        // println!("{:?}", data.system_listeners.keys());

        let keys = data.system_listeners.keys().cloned().collect();

        match data.tx.send(keys) {
            Err(e) => println!("{:?}", e),
            Ok(()) => {}
        };
    }
}
