#![allow(unused)]

mod error;
mod listener;
mod utils;

use error::Result;
use listener::Listener;
use tokio::join;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = Listener::new();

    join!(tokio::spawn(listener.listen()));

    Ok(())
}
