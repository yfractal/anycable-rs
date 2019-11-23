use super::streams::Streams;

use futures::sync::mpsc;

use std::collections::{HashMap, HashSet};

use std::net::SocketAddr;
use tungstenite::protocol::Message;

use std::sync::{Arc, Mutex};

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
    streams: HashSet<Stream>
}

impl Connection {
    pub fn new(sender: Sender, identifiers: String) -> Connection {
        Connection {
            sender: sender,
            identifiers: identifiers,
            streams: HashSet::new(),
        }
    }

    pub fn send_msg(&self, msg: String) {
        let msg = Message::Text(msg.into());
        self.sender.unbounded_send(msg).unwrap();
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
    inner: HashMap<SocketAddr, Connection>,
    addrs: HashSet<SocketAddr>,
}

impl Connections {
    pub fn new() -> Connections {
        Connections {
            inner: HashMap::new(),
            addrs: HashSet::new(),
        }
    }

    pub fn add_conn(&mut self, addr: SocketAddr, connection: Connection) {
        self.inner.insert(addr, connection);
        self.addrs.insert(addr);
    }

    pub fn remove_conn(&mut self, addr: &SocketAddr) {
        self.inner.remove(addr);
        self.addrs.remove(addr);
    }

    pub fn get_conn_identifiers(&self, addr: &SocketAddr) -> String {
        self.inner.get(addr).unwrap().identifiers.clone()
    }

    pub fn send_msg_to_conn(&self, addr: &SocketAddr, msg: String) {
        match self.inner.get(addr) {
            Some(addr) => addr.send_msg(msg),
            _ => (),
        };
    }

    pub fn broadcast(&self, msg: String) {
        for addr in self.addrs.iter() {
            self.send_msg_to_conn(addr, msg.to_string());
        }
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

    pub fn stop_streams(&mut self, strems: Arc<Mutex<Streams>>, addr: SocketAddr, channel: &str) {
        let mut streams_to_delete = Vec::new();
        for stream in self.inner.get(&addr).unwrap().get_streams() {
            strems.lock().unwrap().
                remove_stream(&stream.name, addr, stream.channel.to_string());
            println!("stream loop {:?}", stream);
            if (stream.channel == channel) {
                let stream = Stream::new(stream.name.to_string(), channel.to_string());
                streams_to_delete.push(stream);
            }
        }

        for stream in streams_to_delete.iter() {
            println!("delete stream {:?}", stream);
            self.inner.get_mut(&addr).unwrap().remove_stream(stream.name.to_string(), stream.channel.to_string());
        }
    }

    pub fn get_conn_channels_vec(&self, addr: &SocketAddr) -> Vec<String> {
        let mut channels = Vec::new();
        for stream in self.get_conn_streams(addr).iter() {
            channels.push(stream.channel.to_string());
        }

        channels
    }

}
