use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};

use protos::anycable_grpc::RpcClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use protobuf::RepeatedField;
use protos::anycable::{ConnectionRequest, ConnectionResponse,
                       CommandMessage, CommandResponse,
                       DisconnectRequest, DisconnectResponse,
                       Status};


use futures::Future;
use futures::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;

use tokio::net::TcpListener;
use tungstenite::protocol::Message;

use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::accept_async;
use std::net::SocketAddr;

use tungstenite::handshake::server::{ErrorResponse, Request};
use tungstenite::http::StatusCode;

use serde_json::{Result, Value};
use serde_json::json;

use tokio::prelude::*;

type Sender = mpsc::UnboundedSender<Message>;

struct Connection {
    sender: Sender,
    identifiers: String
}

impl Connection {
    pub fn new(sender: Sender, identifiers: String) -> Connection {
        Connection {
            sender: sender,
            identifiers: identifiers
        }
    }

    pub fn send_msg(&self, msg: String) {
        let msg = Message::Text(msg.into());
        self.sender.unbounded_send(msg).unwrap();
    }
}


#[derive(PartialEq, Eq, Hash, Debug)]
struct Subscriber {
    addr: SocketAddr,
    channel: String
}

impl Subscriber {
    pub fn new(addr: SocketAddr, channel: String) -> Subscriber {
        Subscriber{
            addr: addr,
            channel: channel
        }
    }
}

#[derive(Debug)]
struct Streams {
    streams: HashMap<String, HashSet<Subscriber>>
}

impl Streams {
    pub fn new() -> Streams {
        Streams{
            streams: HashMap::new()
        }
    }

    pub fn get(&self, stream: &str) -> &HashSet<Subscriber> {
        self.streams.get(stream).unwrap()
    }

    pub fn remove(&mut self, stream: &str, addr: SocketAddr, channel: String) -> bool{
        let subscriber = Subscriber::new(addr, channel);
        match self.streams.get_mut(stream) {
            Some(subscribers) => {
                subscribers.remove(&subscriber);
                true
            },
            None => {
                false
            }
        }
    }

    pub fn put(&mut self, stream: &str, addr: SocketAddr, channel: String) {
        let subscriber = Subscriber::new(addr, channel);
        match self.streams.get_mut(stream) {
            Some(subscribers) => {
                subscribers.insert(subscriber);
            },
            None => {
                let mut subscribers = HashSet::new();
                subscribers.insert(subscriber);
                self.streams.insert(stream.to_string(), subscribers);
            }
        }
    }
}

#[derive(Debug)]
struct Subscribers {
    inner: HashMap<Subscriber, HashSet<String>>
}

impl Subscribers {
    pub fn new() -> Subscribers {
        Subscribers {
            inner: HashMap::new()
        }
    }

    pub fn get(&self, addr: SocketAddr, channel: String) -> &HashSet<String> {
        let subscriber = Subscriber::new(addr, channel);
        self.inner.get(&subscriber).unwrap()
    }

    pub fn remove_subscriber(&mut self, addr: SocketAddr, channel: String) {
        let subscriber = Subscriber::new(addr, channel);
        self.inner.remove(&subscriber);
    }

    pub fn put(&mut self, addr: SocketAddr, channel: String, stream: &str) {
        let subscriber = Subscriber::new(addr, channel);
        match self.inner.get_mut(&subscriber) {
            Some(streams) => {
                streams.insert(stream.to_string());
            },
            None => {
                let mut streams = HashSet::new();
                streams.insert(stream.to_string());
                self.inner.insert(subscriber, streams);
            }
        }
    }
}

#[derive(Debug)]
struct AddrToChanels {
    inner: HashMap<SocketAddr, HashSet<String>>
}

impl AddrToChanels {
    pub fn new() -> AddrToChanels {
        AddrToChanels {
            inner: HashMap::new()
        }
    }

    pub fn get(&self, addr: SocketAddr) -> &HashSet<String> {
        self.inner.get(&addr).unwrap()
    }

    pub fn get_vec(&self, addr: SocketAddr) -> Vec<String> {
        let mut channels = Vec::new();
        for c in self.get(addr).iter() {
            channels.push(c.to_string());
        }

        channels
    }

    pub fn remove_addr(&mut self, addr: SocketAddr) {
        self.inner.remove(&addr);
    }

    pub fn remove_channel(&mut self, addr: SocketAddr, channel: String) -> bool {
        match self.inner.get_mut(&addr) {
            Some(channels) => {
                channels.remove(&channel);
                true
            },
            None => {
                false
            }
        }
    }

    pub fn put(&mut self, addr: SocketAddr, channel: String) {
        match self.inner.get_mut(&addr) {
            Some(channels) => {
                channels.insert(channel);
            },
            None => {
                let mut channels = HashSet::new();
                channels.insert(channel);
                self.inner.insert(addr, channels);
            }
        }
    }
}

struct WsServer {
    connections: Arc<Mutex<HashMap<SocketAddr, Connection>>>,
}

impl WsServer {
    pub fn broadcast(&mut self) {
        let msg = Message::Text("Hello WebSocket".into());
        let mut connections = self.connections.lock().unwrap();
        let iter = connections
            .iter_mut()
            .filter(|&(&k, _)| true)
            .map(|(_, v)| v);
        for v in iter {
            v.sender.unbounded_send(msg.clone()).unwrap();
        }
    }

    pub fn send_message(&self, addr: SocketAddr, msg: String) {
        let connections = self.connections.lock().unwrap();
        connections.get(&addr).unwrap().send_msg(msg);
    }
}

struct RedisConsumer {
    connections: Arc<Mutex<HashMap<SocketAddr, Connection>>>,
    msg_queue: mpsc::UnboundedReceiver<String>,
    streams: Arc<Mutex<Streams>>,
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
                    let data: Value = serde_json::from_str(&raw_data.as_str().unwrap().to_string()).unwrap();

                    for sub in self.streams.lock().unwrap().get(stream.as_str().unwrap()).iter() {
                        let connections = self.connections.lock().unwrap();
                        let msg = json!({
                            "identifier": sub.channel,
                            "message": data
                        });

                        connections.get(&sub.addr).unwrap().send_msg(msg.to_string());
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

fn rpc_connect(client: Arc<Mutex<RpcClient>>, headers: HashMap<String, String>) -> ConnectionResponse {
    let mut req = ConnectionRequest::default();
    req.set_path("/cable".to_owned());
    req.set_headers(headers);

    client.lock().unwrap().connect(&req).expect("rpc")
}

fn rpc_disconnect(client: Arc<Mutex<RpcClient>>, id: String, channels: Vec<String>) -> DisconnectResponse {
    let mut req = DisconnectRequest::default();
    req.set_identifiers(id);
    let subscriptions = RepeatedField::from_vec(channels);
    req.set_subscriptions(subscriptions);

    client.lock().unwrap().disconnect(&req).expect("rpc")
}

fn rpc_command(client: Arc<Mutex<RpcClient>>,
               command: String,
               identifiers: String,
               channel: String,
               data: String) -> CommandResponse {

    let mut req = CommandMessage::default();
    req.set_command(command);
    req.set_identifier(channel);
    req.set_connection_identifiers(identifiers);
    req.set_data(data);

    client.lock().unwrap().command(&req).expect("rpc")
}

fn rpc_subscribe(client: Arc<Mutex<RpcClient>>, identifiers: String, channel: String) -> CommandResponse {
    rpc_command(client, "subscribe".to_string(), identifiers, channel, "".to_string())
}

fn rpc_unsubscribe(client: Arc<Mutex<RpcClient>>, identifiers: String, channel: String) -> CommandResponse {
    rpc_command(client, "unsubscribe".to_string(), identifiers, channel, "".to_string())
}

fn rpc_message(client: Arc<Mutex<RpcClient>>, identifiers: String, channel: String, data: String) -> CommandResponse {
    rpc_command(client, "message".to_string(), identifiers, channel, data)
}

pub fn start_ws_server(redis_receiver: mpsc::UnboundedReceiver<String>) -> tokio::executor::Spawn {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:50051");
    let client = Arc::new(Mutex::new(RpcClient::new(ch)));

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:3334".to_string());
    let addr = addr.parse().unwrap();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let connections = Arc::new(Mutex::new(HashMap::new()));
    let streams = Arc::new(Mutex::new(Streams::new()));
    let subscribers = Arc::new(Mutex::new(Subscribers::new()));
    let addr_to_channels = Arc::new(Mutex::new(AddrToChanels::new()));

    let addr_to_header = Arc::new(Mutex::new(HashMap::new()));

    let ws_server = Arc::new(Mutex::new(WsServer{connections: connections.clone()}));

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
        let subscribers_inner = subscribers.clone();
        let subscribers_inner2 = subscribers.clone();
        let addr_to_channels_inner = addr_to_channels.clone();
        let addr_to_channels_inner2 = addr_to_channels.clone();
        let addr_to_header_inner = addr_to_header.clone();

        let ws_server_inner = ws_server.clone();
        let client_inner = client.clone();

        let callback = move |req: &Request| {
            let mut headers = HashMap::new();

            for &(ref header, ref val) in req.headers.iter() {
                headers.insert(header.to_string(), std::str::from_utf8(val).unwrap().to_string());
            }

            addr_to_header_inner.lock().unwrap().insert(addr, headers);

            Ok(None)
        };

        let addr_to_header_inner = addr_to_header.clone();

        let f = accept_hdr_async(stream, callback)
            .and_then(move |mut ws_stream| {
                println!("New WebSocket connection: {}", addr);

                let headers = addr_to_header_inner.lock().unwrap().remove(&addr).unwrap();
                let client_inner2 = client_inner.clone();
                let client_inner3 = client_inner.clone();
                let reply = rpc_connect(client_inner, headers);

                match reply.get_status() {
                    Status::SUCCESS => {
                        let (tx, rx) = futures::sync::mpsc::unbounded();

                        let connection = Connection::new(tx, reply.get_identifiers().to_string());

                        connections_inner.lock().unwrap().insert(addr, connection);

                        for t in reply.get_transmissions().iter() {
                            ws_server_inner.lock().unwrap().send_message(addr, t.to_string());
                        }

                        let (sink, stream) = ws_stream.split();

                        let connections = connections_inner.clone();

                        let ws_reader = stream.for_each(move |mut message: Message| {
                            let data = message.to_text().unwrap();
                            if data != "" {
                                let v: Value = serde_json::from_str(data).unwrap();
                                let command = &v["command"];

                                if command == "subscribe" {
                                    let channel = &v["identifier"];
                                    let identifiers = &connections.lock().unwrap().get(&addr).unwrap().identifiers.clone();

                                    let reply = rpc_subscribe(client_inner2.clone(),
                                                              identifiers.to_string(),
                                                              channel.as_str().unwrap().to_string());

                                    for t in reply.get_transmissions().iter() {
                                        ws_server_inner.lock().unwrap().send_message(addr, t.to_string());
                                    }
                                } else if command == "unsubscribe" {
                                    let channel = &v["identifier"];
                                    let identifiers = &connections.lock().unwrap().get(&addr).unwrap().identifiers.clone();

                                    let reply = rpc_unsubscribe(client_inner2.clone(),
                                                                identifiers.to_string(),
                                                                channel.as_str().unwrap().to_string());

                                    for t in reply.get_transmissions().iter() {
                                        ws_server_inner.lock().unwrap().send_message(addr, t.to_string());
                                    }

                                    for stream in subscribers_inner.lock().unwrap().get(addr, channel.as_str().unwrap().to_string()).iter() {
                                        streams_inner.lock().unwrap().remove(stream, addr, channel.as_str().unwrap().to_string());
                                    }

                                    subscribers_inner.lock().unwrap().remove_subscriber(addr, channel.as_str().unwrap().to_string());
                                    addr_to_channels_inner.lock().unwrap().remove_channel(addr, channel.as_str().unwrap().to_string());
                                } else if command == "message" {
                                    let channel = &v["identifier"];
                                    let data = &v["data"];
                                    let identifiers = &connections.lock().unwrap().get(&addr).unwrap().identifiers.clone();

                                    let reply = rpc_message(client_inner2.clone(),
                                                            identifiers.to_string(),
                                                            channel.as_str().unwrap().to_string(),
                                                            data.as_str().unwrap().to_string());

                                    for t in reply.get_transmissions().iter() {
                                        ws_server_inner.lock().unwrap().send_message(addr, t.to_string());
                                    }

                                    for stream in reply.get_streams().iter() {
                                        streams_inner.lock().unwrap().put(stream, addr, channel.as_str().unwrap().to_string());
                                        subscribers_inner.lock().unwrap().put(addr, channel.as_str().unwrap().to_string(), stream);
                                    }

                                    addr_to_channels_inner.lock().unwrap().put(addr, channel.as_str().unwrap().to_string());
                                }
                            }

                            Ok(())
                        });

                        let ws_writer = rx.fold(sink, |mut sink, msg| {
                            sink.start_send(msg).unwrap();
                            Ok(sink)
                        });

                        let connection = ws_reader
                            .map(|_| ())
                            .map_err(|_| ())
                            .select(ws_writer.map(|_| ()).map_err(|_| ()));

                        tokio::spawn(connection.then(move |_| {
                            rpc_disconnect(client_inner3,
                                           connections_inner.lock().unwrap().get(&addr).unwrap().identifiers.to_string(),
                                           addr_to_channels_inner2.lock().unwrap().get_vec(addr));

                            connections_inner.lock().unwrap().remove(&addr);

                            let channels = addr_to_channels_inner2.lock().unwrap().get_vec(addr);

                            for channel in channels.iter() {
                                for stream in subscribers_inner2.lock().unwrap().get(addr, channel.to_string()).iter() {
                                    streams_inner2.lock().unwrap().remove(stream, addr, channel.to_string());
                                }

                                subscribers_inner2.lock().unwrap().remove_subscriber(addr, channel.to_string());
                            }

                            addr_to_channels_inner2.lock().unwrap().remove_addr(addr);
                            println!("Connection {} closed.", addr);
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

    tokio::spawn(srv)
}
