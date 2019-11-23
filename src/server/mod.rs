pub mod streams;
pub mod connections;
mod rpc;
mod server;

use server::Server;

use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

use futures::Future;
use futures::Sink;
use futures::stream::Stream;

use tokio::net::TcpListener;
use tokio::timer::Interval;

use tungstenite::protocol::Message;
use tungstenite::handshake::server::{Request};
use tokio_tungstenite::accept_hdr_async;

use serde_json::Value;
use serde_json::json;

use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

// redis
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use redis_async::resp::FromResp;


use tokio::prelude::*;

pub fn start_ws_server() -> tokio::executor::Spawn {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:3334".to_string());
    let addr = addr.parse().unwrap();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let server = Arc::new(Mutex::new(Server::new("localhost:50051")));

    let server1 = server.clone();
    let redis_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6379);
    let redis_loop = redis_async::client::pubsub_connect(&redis_addr).
        and_then(move |connection| connection.subscribe("__anycable__")).
        map_err(|e| eprintln!("error: cannot receive messages. error={:?}", e)).
        and_then(move |msgs| {
            msgs.for_each(move |message| {
                let v = String::from_resp(message).unwrap();
                let v: Value = serde_json::from_str(&v).unwrap();
                let stream = &v["stream"];
                let raw_data = &v["data"];
                let data: Value = serde_json::from_str(raw_data.as_str().unwrap()).unwrap();

                server1.lock().unwrap().
                    broadcast_to_stream(stream.as_str().unwrap(), data);
                future::ok(())
            }).map_err(|e| eprintln!("error: redis subscribe stoped, error {:?}", e))
        });

    tokio::spawn(redis_loop);

    let server_inner = server.clone();

    let addr_to_header = Arc::new(Mutex::new(HashMap::new()));

    let srv = socket.incoming().for_each(move |stream| {
        let addr = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        println!("Peer address: {}", addr);

        let server_inner = server.clone();
        let server_inner2 = server.clone();

        let addr_to_header_inner = addr_to_header.clone();

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

                let (tx, rx) = futures::sync::mpsc::unbounded();

                match server_inner.lock().unwrap().
                    connect(addr, headers, tx) {
                    true => {
                        let (sink, stream) = ws_stream.split();

                        let server = server_inner.clone();

                        let ws_reader = stream.for_each(move |message: Message| {
                            let data = message.to_text().unwrap();

                            server.lock().unwrap().
                                receive_message(addr, data);

                            Ok(())
                        });

                        let ws_writer = rx.fold(sink, |mut sink, msg| {
                            if msg.to_text().unwrap() == "disconnect!" {
                                sink.close().unwrap();
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
                            server_inner2.clone().lock().unwrap().
                                disconnect(addr);

                            Ok(())
                        }));
                    },
                    _ => {
                        ws_stream.close().unwrap();
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

            server_inner.lock().unwrap().broadcast(msg.to_string());

            Ok(())
        }).map_err(|_e| ());

    tokio::spawn(ping_interval);

    tokio::spawn(srv)
}
