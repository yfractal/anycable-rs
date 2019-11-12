use futures::sync::mpsc;

use std::collections::{HashMap, HashSet};

use std::net::SocketAddr;
use tungstenite::protocol::Message;

type Sender = mpsc::UnboundedSender<Message>;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Stream {
    pub name: String,
    pub channel: String
}


impl Stream {
    pub fn new(name: String, channel: String) -> Stream {
        Stream {
            name: name,
            channel: channel
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    pub sender: Sender,
    pub identifiers: String,
    channels: HashSet<String>,
    streams: HashSet<Stream>
}

impl Connection {
    pub fn new(sender: Sender, identifiers: String) -> Connection {
        Connection {
            sender: sender,
            identifiers: identifiers,
            channels: HashSet::new(),
            streams: HashSet::new(),
        }
    }

    pub fn send_msg(&self, msg: String) {
        let msg = Message::Text(msg.into());
        self.sender.unbounded_send(msg).unwrap();
    }

    pub fn get_channels(&self) -> &HashSet<String> {
        &self.channels
    }

    pub fn get_channels_vec(&self) -> Vec<String> {
        let mut channels = Vec::new();
        for c in self.channels.iter() {
            channels.push(c.to_string());
        }

        channels
    }

    pub fn remove_channel(&mut self, channel: String) {
        self.channels.remove(&channel);
    }

    pub fn add_channel(&mut self, channel: String) {
        self.channels.insert(channel);
    }

    pub fn add_stream(&mut self, stream_name: String, channel: String) {
        let stream = Stream::new(stream_name, channel);
        self.streams.insert(stream);
    }

    pub fn remove_stream(&mut self, stream_name: String, channel: String) {
        let stream = Stream::new(stream_name, channel);
        self.streams.remove(&stream);
    }

    pub fn get_streams(&self) -> &HashSet<Stream> {
        &self.streams
    }
}

#[derive(Debug)]
pub struct Connections {
    inner: HashMap<SocketAddr, Connection>
}

impl Connections {
    pub fn new() -> Connections {
        Connections {
            inner: HashMap::new(),
        }
    }

    pub fn create_connection(&mut self, addr: SocketAddr, sender: Sender, identifiers: String) {
        let connection = Connection::new(sender, identifiers);
        self.inner.insert(addr, connection);
    }

    pub fn add_connection(&mut self, addr: SocketAddr, connection: Connection) {
        self.inner.insert(addr, connection);
    }

    pub fn remove_connection(&mut self, addr: &SocketAddr) {
        self.inner.remove(addr);
    }

    pub fn get_connection_identifiers(&self, addr: &SocketAddr) -> String {
        self.inner.get(addr).unwrap().identifiers.clone()
    }

    pub fn send_msg_to_connection(&self, addr: &SocketAddr, msg: String) {
        self.inner.get(addr).unwrap().send_msg(msg);
    }


    pub fn add_channel_to_conn(&mut self, addr: &SocketAddr, channel: String) {
        self.inner.get_mut(addr).unwrap().add_channel(channel);
    }

    pub fn remove_conn_channel(&mut self, addr: &SocketAddr, channel: String) {
        self.inner.get_mut(addr).unwrap().remove_channel(channel);
    }

    pub fn get_conn_channels_vec(&self, addr: &SocketAddr) -> Vec<String> {
        self.inner.get(addr).unwrap().get_channels_vec()
    }

    pub fn add_stream_to_conn(&mut self, addr: &SocketAddr, stream: String, channel: String) {
        self.inner.get_mut(addr).unwrap().add_stream(stream, channel);
    }

    pub fn remove_conn_stream(&mut self, addr: &SocketAddr, stream: String, channel: String) {
        self.inner.get_mut(addr).unwrap().remove_stream(stream, channel);
    }

    pub fn get_conn_streams(&self, addr: &SocketAddr) -> &HashSet<Stream> {
        self.inner.get(addr).unwrap().get_streams()
    }
}
