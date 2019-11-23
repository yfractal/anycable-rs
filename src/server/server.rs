use super::Streams;
use super::Connections;

use std::collections::HashMap;
use std::net::SocketAddr;

use protos::anycable::{Status, CommandResponse, ConnectionResponse};

pub struct Server {
    connections:  Connections,
    streams: Streams,
    rpc_client: super::rpc::Client
}

impl Server {
    pub fn new(host: &str) -> Server {
        Server {
            connections: Connections::new(),
            streams: Streams::new(),
            rpc_client: super::rpc::Client::new(host),
        }
    }

    pub fn connect(&self, addr: SocketAddr, headers: HashMap<String, String>) -> bool {
        let resp =  self.rpc_client.connect(headers);
        match resp.get_status() {
            Status::SUCCESS => {
                true
            },
            _ => false
        }
    }

    // pub fn connected(addr: SocketAddr, sender: Sender, rep: ConnectionResponse) -> {
    //     let connection = super::connections::Connection::new(sender, rep.get_identifiers().to_string());
    //     connections.lock().unwrap().add_conn(addr, connection);

    //     for t in respone.get_transmissions().iter() {
    //         connections.lock().unwrap().
    //             send_msg_to_conn(&addr, t.to_string());
    //     }

    // }
}

// fn connect() -> {

// }

// fn connected() -> {

// }

// fn receive_message() -> {

// }

// fn close_connection() -> {

// }
