#![allow(unused)]

mod error;
mod listener;
mod utils;

use error::Result;
use listener::listen;
use tokio::join;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = listen();

    let mut rx = listener.rx.clone();

    loop {
        println!("{:?}", *rx.borrow_and_update());
        if rx.changed().await.is_err() {
            break;
        }
    }

    join!(listener.handle);

    Ok(())
}
