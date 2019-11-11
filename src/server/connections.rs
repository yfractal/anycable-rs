use futures::sync::mpsc;

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

pub fn hello() {
    println!("hello word");
}
