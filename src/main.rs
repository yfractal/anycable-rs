// #![deny(warnings, rust_2018_idioms)]
mod ws_server;
mod server;
// extern crate websocket;

use std::thread;
// use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use futures::future::lazy;
use futures::sync::mpsc;
use tokio::prelude::*;

use bytes::Bytes;

use std::collections::HashMap;
use std::env;
use std::io::{Error, ErrorKind};

use futures::stream::Stream;
use futures::Future;
use tokio::net::TcpListener;
use tungstenite::protocol::Message;

use tokio_tungstenite::accept_async;

fn do_sub(tx: mpsc::UnboundedSender<String>) -> () {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();

    let mut con = client.get_connection().unwrap();
    let mut pubsub = con.as_pubsub();

    pubsub.subscribe("__anycable__").unwrap();

    loop {
        let msg = pubsub.get_message().unwrap();
        let payload : String = msg.get_payload().unwrap();

        // TODO: consume here directly, no need send to queue
        tx.unbounded_send(payload).unwrap();
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::unbounded();
    // it's not clever to use thread with tokio :(.
    // but it seems like that tokio doesn't have good way to handle task priority.

    let redis_handler = thread::spawn(|| {
        do_sub(tx);
    });

    tokio::run(lazy(|| {
        // ws_server::start_ws_server(rx);
        server::start_ws_server(rx)
    }));

    redis_handler.join().unwrap();
    Ok(())
}
