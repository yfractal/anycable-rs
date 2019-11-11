use futures::sync::mpsc;

use std::collections::{HashMap, HashSet};

use std::net::SocketAddr;
use tungstenite::protocol::Message;

type Sender = mpsc::UnboundedSender<Message>;

pub struct Connection {
    pub sender: Sender,
    pub identifiers: String
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
}
