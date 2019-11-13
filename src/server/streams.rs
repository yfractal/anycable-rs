use std::net::SocketAddr;

use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Hash, Debug)]
// TODO: check stream.channel, stream.channel will remove variable or not :(
pub struct Stream {
    pub addr: SocketAddr,
    pub channel: String,
}

impl Stream {
    pub fn new(addr: SocketAddr, channel: String) -> Stream {
        Stream {
            addr: addr,
            channel: channel
        }
    }
}

#[derive(Debug)]
pub struct Streams {
    // map stream name to stream
    inner: HashMap<String, HashSet<Stream>>
}

impl Streams {
    pub fn new() -> Streams {
        Streams {
            inner: HashMap::new(),
        }
    }

    pub fn put_stream(&mut self, stream_name: &str, addr: SocketAddr, channel: String) {
        let stream = Stream::new(addr, channel);
        match self.inner.get_mut(stream_name) {
            Some(streams) => {
                streams.insert(stream);
            },
            None => {
                let mut streams = HashSet::new();
                streams.insert(stream);
                self.inner.insert(stream_name.to_string(), streams);
            }
        }
    }

    pub fn remove_stream(&mut self, stream_name: &str, addr: SocketAddr, channel: String) -> bool {
        let stream = Stream::new(addr, channel);
        match self.inner.get_mut(stream_name) {
            Some(streams) => {
                streams.remove(&stream);
                true
            },
            None => {
                false
            }
        }
    }

    pub fn get(&self, stream: &str) -> &HashSet<Stream> {
        self.inner.get(stream).unwrap()
    }
}
