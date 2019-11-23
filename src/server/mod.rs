pub mod streams;

pub mod connections;

mod rpc;

use connections::Connection;
use connections::Connections;
use streams::Streams;

use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};

use protos::anycable::{Status, CommandResponse, ConnectionResponse};

use futures::Future;
use futures::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;

use tokio::net::TcpListener;
use tokio::timer::Interval;

use tungstenite::protocol::Message;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::accept_async;

use std::net::SocketAddr;

use tungstenite::handshake::server::{ErrorResponse, Request};
use tungstenite::http::StatusCode;

use serde_json::{Result, Value};
use serde_json::json;

use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};


use tokio::prelude::*;

type Sender = mpsc::UnboundedSender<Message>;

#[derive(Debug)]
struct RedisConsumer {
    connections: Arc<Mutex<connections::Connections>>,
    msg_queue: mpsc::UnboundedReceiver<String>,
    streams: Arc<Mutex<streams::Streams>>,
}

impl Future for RedisConsumer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        const TICK: usize = 10;

        for i in 0..TICK {
            match self.msg_queue.poll().unwrap() {
                Async::Ready(Some(v)) => {
                    let v: Value = serde_json::from_str(&v).unwrap();
                    let stream = &v["stream"];
                    let raw_data = &v["data"];
                    let data: Value = serde_json::from_str(raw_data.as_str().unwrap()).unwrap();
                    for stream in self.streams.lock().unwrap().get(stream.as_str().unwrap()).iter() {
                        let msg = json!({
                            "identifier": stream.channel,
                            "message": data
                        });
                        self.connections.lock().unwrap().
                            send_msg_to_conn(&stream.addr, msg.to_string());
                    }

                    if i + 1 == TICK {
                        task::current().notify();
                    }
                }
                _ => break,
            }
        }

        Ok(Async::NotReady)
    }
}

fn stop_channel_streams(connections: Arc<Mutex<Connections>>,
                        streams: Arc<Mutex<Streams>>,
                        addr: SocketAddr,
                        channel: &str) {
    connections.lock().unwrap().stop_streams(streams.clone(), addr, &channel);
}

fn start_channel_streams(
    connections: Arc<Mutex<Connections>>,
    streams: Arc<Mutex<Streams>>,
    addr: SocketAddr,
    channel: String,
    respone: CommandResponse) {

    for stream in respone.get_streams().iter() {
        streams.lock().unwrap().
            put_stream(stream, addr, channel.to_string());

        connections.lock().unwrap().add_stream_to_conn(&addr, stream.to_string(), channel.to_string());
    }
}

fn handle_rpc_command_resp(
    connections: Arc<Mutex<Connections>>,
    streams: Arc<Mutex<Streams>>,
    addr: SocketAddr,
    channel: String,
    respone: CommandResponse) {

    for t in respone.get_transmissions().iter() {
        connections.lock().unwrap().
            send_msg_to_conn(&addr, t.to_string());
    }

    if respone.get_disconnect() {
        connections.lock().unwrap().send_msg_to_conn(&addr, "disconnect!".to_string());

        return ();
    }
    if respone.get_stop_streams() {
        // follow erlycable implemention, hope it's not a bug, more info as below:
        // https://github.com/anycable/erlycable/blob/master/src/erlycable_server.erl#L242-L244
        stop_channel_streams(connections.clone(), streams.clone(), addr, &channel);
        start_channel_streams(connections, streams, addr, channel, respone);
    } else {
        start_channel_streams(connections, streams, addr, channel, respone);
    }
}

fn handle_rpc_conn_resp(
    connections: Arc<Mutex<Connections>>,
    streams: Arc<Mutex<Streams>>,
    addr: SocketAddr,
    sender: Sender,
    respone: ConnectionResponse) {

    let connection = self::connections::Connection::new(sender, respone.get_identifiers().to_string());
    connections.lock().unwrap().add_conn(addr, connection);

    for t in respone.get_transmissions().iter() {
        connections.lock().unwrap().
            send_msg_to_conn(&addr, t.to_string());
    }
}

fn handle_user_command(
    connections: Arc<Mutex<Connections>>,
    streams: Arc<Mutex<Streams>>,
    rpc_client: Arc<Mutex<rpc::Client>>,
    addr: SocketAddr,
    data: &str) {
    if data != "" {
        let v: Value = serde_json::from_str(data).unwrap();
        let command = &v["command"];
        let mut command_data = "";

        if command == "message" {
            command_data = &v["data"].as_str().unwrap();
        }

        let channel = &v["identifier"];
        let identifiers = connections.lock().unwrap().get_conn_identifiers(&addr);

        let reply = rpc_client.lock().unwrap().
            command(command.as_str().unwrap().to_string(),
                    identifiers.to_string(),
                    channel.as_str().unwrap().to_string(),
                    command_data.to_string());

        handle_rpc_command_resp(connections.clone(),
                                streams.clone(),
                                addr, channel.as_str().unwrap().to_string(),
                                reply);
    } else {
        println!("handle_rpc_command: empty data");
    }
}

fn close_connection(
    connections: Arc<Mutex<Connections>>,
    streams: Arc<Mutex<Streams>>,
    addr: SocketAddr,
    rpc_client: Arc<Mutex<rpc::Client>>) {

    let identifiers = connections.lock().unwrap().get_conn_identifiers(&addr);
    let channels = connections.lock().unwrap().get_conn_channels_vec(&addr);
    let reply = rpc_client.lock().unwrap().disconnect(identifiers, channels);

    for stream in connections.lock().unwrap().get_conn_streams(&addr).iter() {
        streams.lock().unwrap().remove_stream(&stream.name, addr, stream.channel.to_string());
    }

    connections.lock().unwrap().remove_conn(&addr);
    println!("Connection {} closed.", addr);
}

pub fn start_ws_server(redis_receiver: mpsc::UnboundedReceiver<String>) -> tokio::executor::Spawn {
    let rpc_client = Arc::new(Mutex::new(rpc::Client::new("localhost:50051")));

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:3334".to_string());
    let addr = addr.parse().unwrap();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let connections = Arc::new(Mutex::new(connections::Connections::new()));
    let connections_inner = connections.clone();
    let streams = Arc::new(Mutex::new(streams::Streams::new()));

    let addr_to_header = Arc::new(Mutex::new(HashMap::new()));

    let rerdis_consumer = RedisConsumer{
        connections: connections.clone(),
        msg_queue: redis_receiver,
        streams: streams.clone(),
    };
    tokio::spawn(rerdis_consumer);

    let srv = socket.incoming().for_each(move |stream| {
        let addr = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("Peer address: {}", addr);

        let connections_inner = connections.clone();

        let streams_inner = streams.clone();
        let streams_inner2 = streams.clone();

        let addr_to_header_inner = addr_to_header.clone();

        let rpc_client_inner = rpc_client.clone();

        let callback = move |req: &Request| {
            let mut headers = HashMap::new();

            for &(ref header, ref val) in req.headers.iter() {
                headers.insert(header.to_string(), std::str::from_utf8(val).unwrap().to_string());
            }

            addr_to_header_inner.lock().unwrap().insert(addr, headers);

            let extra_headers = vec![
                (String::from("Sec-WebSocket-Protocol"), String::from("actioncable-v1-json")),
            ];

            Ok(Some(extra_headers))
        };

        let addr_to_header_inner = addr_to_header.clone();

        let f = accept_hdr_async(stream, callback)
            .and_then(move |mut ws_stream| {
                println!("New WebSocket connection: {}", addr);

                let headers = addr_to_header_inner.lock().unwrap().remove(&addr).unwrap();
                let rpc_client = rpc_client_inner.clone();

                let reply = rpc_client_inner.lock().unwrap().
                    connect(headers);

                match reply.get_status() {
                    Status::SUCCESS => {
                        let (tx, rx) = futures::sync::mpsc::unbounded();

                        handle_rpc_conn_resp(connections_inner.clone(),
                                             streams_inner.clone(),
                                             addr,
                                             tx,
                                             reply);

                        let (mut sink, stream) = ws_stream.split();

                        let connections = connections_inner.clone();

                        let ws_reader = stream.for_each(move |mut message: Message| {
                            let data = message.to_text().unwrap();

                            handle_user_command(connections.clone(),
                                                streams_inner.clone(),
                                                rpc_client_inner.clone(),
                                                addr,
                                                data);
                            Ok(())
                        });

                        let ws_writer = rx.fold(sink, |mut sink, msg| {
                            if msg.to_text().unwrap() == "disconnect!" {
                                sink.close();
                            } else {
                                sink.start_send(msg).unwrap();
                            }

                            Ok(sink)
                        });

                        let connection = ws_reader
                            .map(|_| ())
                            .map_err(|_| ())
                            .select(ws_writer.map(|_| ()).map_err(|_| ()));

                        tokio::spawn(connection.then(move |_| {
                            close_connection(connections_inner.clone(),
                                             streams_inner2.clone(),
                                             addr,
                                             rpc_client.clone());

                            Ok(())
                        }));
                    },
                    _ => {
                        ws_stream.close();
                    }
                }
                Ok(())
            })
            .map_err(|e| {
                println!("Error during the websocket handshake occurred: {}", e);
            });

        tokio::spawn(f);

        // always return ok
        Ok(())
    }).map_err(|_e| ());

    let ping_interval = Interval::new_interval(Duration::from_millis(3000))
        .for_each(move |_| {
            let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let msg = json!({
                "type": "ping",
                "message": since.as_secs(),
            });
            connections_inner.lock().unwrap().broadcast(msg.to_string());

            Ok(())
        }).map_err(|e| ());

    tokio::spawn(ping_interval);

    tokio::spawn(srv)
}
