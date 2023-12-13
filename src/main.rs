pub mod listener;
pub mod system_listener;

#[tokio::main]
async fn main() {
    let listen_simstatus_handle = tokio::spawn(listener::listen_simstatus());

    let _ = listen_simstatus_handle.await;
}
