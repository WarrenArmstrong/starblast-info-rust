mod dto;
mod system_listener;

use dto::ServerDTO;
use std::{alloc::System, sync::Arc};
use system_listener::SystemListener;
use tokio::{
    sync::watch::{self, Receiver, Sender},
    task::JoinHandle,
    time,
};

use std::collections::HashMap;

use reqwest::Error;

use self::dto::SimStatusDTO;

pub struct Starblast {}

pub struct Listener {
    tx: Sender<Starblast>,
    system_listeners: HashMap<i32, JoinHandle<()>>,
}

impl Listener {
    pub fn new() -> Self {
        let (tx, mut _rx) = watch::channel(Starblast {});

        Self {
            tx: tx,
            system_listeners: HashMap::new(),
        }
    }

    pub fn subscribe(self) -> Receiver<Starblast> {
        self.tx.subscribe()
    }

    pub async fn listen(mut self) {
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

            println!("parsed simstatus {:?}", self.system_listeners.keys());

            self.system_listeners.retain(|k, v| {
                if v.is_finished() {
                    println!("{} deleted system listener", k);
                };
                !v.is_finished()
            });

            // simstatus.servers.iter()

            for server in simstatus.servers {
                for system in server.systems {
                    if system.mode == "team" && !self.system_listeners.contains_key(&system.id) {
                        let system_listener =
                            SystemListener::new(system.id, server.address.clone());
                        let handle = tokio::spawn(system_listener.listen());

                        self.system_listeners.insert(system.id, handle);

                        println!("{} started system listener", system.id);
                    }
                }
            }

            // retain

            // create new
        }
    }
}
