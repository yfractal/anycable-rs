use std::sync::Arc;
use std::collections::{HashMap};

use protos::anycable_grpc::RpcClient;
use grpcio::{ChannelBuilder, EnvBuilder};

use protos::anycable::{ConnectionRequest, ConnectionResponse,
                       CommandMessage, CommandResponse,
                       DisconnectRequest, DisconnectResponse};

use protobuf::RepeatedField;

pub struct Client {
    client: RpcClient,
}

impl Client {
    pub fn new(address: &str) -> Client {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(address);
        let client = RpcClient::new(ch);

        Client {
            client: client,
        }
    }

    pub fn connect(&self, headers: HashMap<String, String>) -> ConnectionResponse {
        let mut req = ConnectionRequest::default();
        req.set_path("/cable".to_owned());
        req.set_headers(headers);

        self.client.connect(&req).expect("rpc")
    }

    pub fn command(&self,
                   command: String,
                   identifiers: String,
                   channel: String,
                   data: String) -> CommandResponse {

        let mut req = CommandMessage::default();
        req.set_command(command);
        req.set_identifier(channel);
        req.set_connection_identifiers(identifiers);
        req.set_data(data);

        self.client.command(&req).expect("rpc")
    }

    pub fn disconnect(&self, id: String, channels: Vec<String>) -> DisconnectResponse {
        let mut req = DisconnectRequest::default();
        req.set_identifiers(id);
        let subscriptions = RepeatedField::from_vec(channels);
        req.set_subscriptions(subscriptions);

        self.client.disconnect(&req).expect("rpc")
    }
}
