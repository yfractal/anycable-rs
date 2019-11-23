use super::Streams;
use super::Connections;

use std::collections::HashMap;
use std::net::SocketAddr;

use protos::anycable::{Status, CommandResponse, ConnectionResponse};

use serde_json::{Result, Value};
use serde_json::json;

use futures::sync::mpsc;
use tungstenite::protocol::Message;

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
        let reply = self.rpc_client.disconnect(identifiers, channels);

        for stream in self.connections.get_conn_streams(&addr).iter() {
            self.streams.remove_stream(&stream.name, addr, stream.channel.to_string());
        }

        self.connections.remove_conn(&addr);
        println!("Connection {} closed.", addr);
    }

    fn stop_channel_streams(
        &mut self,
        addr: SocketAddr,
        channel: &str) {

        let mut streams_to_delete = Vec::new();

        for stream in self.connections.get_conn_streams(&addr) {
            self.streams.remove_stream(&stream.name, addr, stream.channel.to_string());

            if (stream.channel == channel) {
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
