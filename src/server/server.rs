use super::streams::Streams;
use super::connections::Connections;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;

use protos::anycable::{Status, CommandResponse, ConnectionResponse};

use serde_json::Value;
use serde_json::json;

use futures::sync::mpsc;
use tungstenite::protocol::Message;

use redis_async::resp::FromResp;

use tokio::timer::Interval;
use tokio::prelude::*;

type Sender = mpsc::UnboundedSender<Message>;

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

    pub fn connect(&mut self,
                   addr: SocketAddr,
                   headers: HashMap<String, String>,
                   sender: Sender) -> bool {
        let response = self.rpc_client.connect(headers);

        match response.get_status() {
            Status::SUCCESS => {
                self.handle_rpc_conn_resp(addr, sender, response);
                true
            },
            _ => false
        }
    }

    pub fn receive_message(&mut self, addr: SocketAddr, message: &str) {
        if message != "" {
            let v: Value = serde_json::from_str(message).unwrap();
            let command = &v["command"];
            let mut command_data = "";

            if command == "message" {
                command_data = &v["data"].as_str().unwrap();
            }

            let channel = &v["identifier"];
            let identifiers = self.connections.get_conn_identifiers(&addr);

            let reply = self.rpc_client.
                command(command.as_str().unwrap().to_string(),
                        identifiers.to_string(),
                        channel.as_str().unwrap().to_string(),
                        command_data.to_string());

            self.handle_rpc_command_resp(addr,
                                         channel.as_str().unwrap().to_string(),
                                         reply);
        } else {
            println!("handle_rpc_command: empty message");
        }
    }

    pub fn disconnect(&mut self, addr: SocketAddr) {
        let identifiers = self.connections.get_conn_identifiers(&addr);
        let channels = self.connections.get_conn_channels_vec(&addr);
        let _reply = self.rpc_client.disconnect(identifiers, channels);

        for stream in self.connections.get_conn_streams(&addr).iter() {
            self.streams.remove_stream(&stream.name, addr, stream.channel.to_string());
        }

        self.connections.remove_conn(&addr);
        println!("Connection {} closed.", addr);
    }

    pub fn broadcast_to_stream(
        &self,
        stream_name: &str,
        message: Value) {

        for stream in self.streams.get(stream_name).iter() {
            let msg = json!({
                "identifier": stream.channel,
                "message": message
            });

            self.connections.send_msg_to_conn(&stream.addr, msg.to_string());
        }
    }

    pub fn broadcast(&self, message: String) {
        self.connections.broadcast(message);
    }

    fn stop_channel_streams(
        &mut self,
        addr: SocketAddr,
        channel: &str) {

        let mut streams_to_delete = Vec::new();

        for stream in self.connections.get_conn_streams(&addr) {
            self.streams.remove_stream(&stream.name, addr, stream.channel.to_string());

            if stream.channel == channel {
                streams_to_delete.push(stream.name.to_string());
            }
        }

        for stream in streams_to_delete.iter() {
            self.connections.remove_conn_stream(&addr, stream.to_string(), channel.to_string());
        }
    }

    fn start_channel_streams(
        &mut self,
        addr: SocketAddr,
        channel: String,
        respone: CommandResponse) {

        for stream in respone.get_streams().iter() {
            self.streams.put_stream(stream, addr, channel.to_string());

            self.connections.add_stream_to_conn(&addr, stream.to_string(), channel.to_string());
        }
    }

    fn handle_rpc_conn_resp(
        &mut self,
        addr: SocketAddr,
        sender: Sender,
        respone: ConnectionResponse) {

        let connection = super::connections::Connection::new(sender, respone.get_identifiers().to_string());
        self.connections.add_conn(addr, connection);

        for t in respone.get_transmissions().iter() {
            self.connections.
                send_msg_to_conn(&addr, t.to_string());
        }
    }

    fn handle_rpc_command_resp(
        &mut self,
        addr: SocketAddr,
        channel: String,
        respone: CommandResponse) {

        for t in respone.get_transmissions().iter() {
            self.connections.send_msg_to_conn(&addr, t.to_string());
        }

        if respone.get_disconnect() {
            self.connections.send_msg_to_conn(&addr, "disconnect!".to_string());

            return ();
        }

        if respone.get_stop_streams() {
            // follow erlycable implemention, hope it's not a bug, more info as below:
            // https://github.com/anycable/erlycable/blob/master/src/erlycable_server.erl#L242-L244
            self.stop_channel_streams(addr, &channel);
            self.start_channel_streams(addr, channel, respone);
        } else {
            self.start_channel_streams(addr, channel, respone);
        }
    }
}

fn start_ping_task(server: Arc<Mutex<Server>>) {
    let ping_interval = Interval::new_interval(Duration::from_millis(3000))
        .for_each(move |_| {
            let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let msg = json!({
                "type": "ping",
                "message": since.as_secs(),
            });

            server.lock().unwrap().broadcast(msg.to_string());

            Ok(())
        }).map_err(|_e| ());

    tokio::spawn(ping_interval);
}

fn start_redis_task(
    server: Arc<Mutex<Server>>,
    addr: SocketAddr,
    topic: String) {
    let redis_loop = redis_async::client::pubsub_connect(&addr).
        and_then(move |connection| connection.subscribe(&topic)).
        map_err(|e| eprintln!("error: cannot receive messages. error={:?}", e)).
        and_then(move |msgs| {
            msgs.for_each(move |message| {
                let v = String::from_resp(message).unwrap();
                let v: Value = serde_json::from_str(&v).unwrap();
                let stream = &v["stream"];
                let raw_data = &v["data"];
                let data: Value = serde_json::from_str(raw_data.as_str().unwrap()).unwrap();

                server.lock().unwrap().
                    broadcast_to_stream(stream.as_str().unwrap(), data);
                future::ok(())
            }).map_err(|e| eprintln!("error: redis subscribe stoped, error {:?}", e))
        });

    tokio::spawn(redis_loop);
}

pub fn start(
    redis_addr: SocketAddr,
    redis_topic: &str,
    rpc_host: &str) -> Arc<Mutex<Server>> {

    // create server struct
    let server = Arc::new(Mutex::new(Server::new(rpc_host)));

    // start task
    start_redis_task(server.clone(), redis_addr, redis_topic.to_string());
    start_ping_task(server.clone());

    server
}
