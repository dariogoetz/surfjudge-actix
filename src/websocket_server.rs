use crate::logging::LOG;

use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use serde_json::json;

use slog::{debug, info};



#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug, Hash, Copy)]
struct ClientID(usize);

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    channel: String,
    message: String,
}

pub struct WebSocketServer {
    // Recipient is an actix actor
    sessions: HashMap<ClientID, Recipient<WSMessage>>,
    channels: HashMap<String, HashSet<ClientID>>,
    counter: usize,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            channels: HashMap::new()
        }
    }

    fn send_channel(&self, channel: &str, message: &str) {
        let message = json!({"channel": channel, "message": message});

        if let Some(sessions) = self.channels.get(channel) {
            info!(
                LOG,
                "Sending message to {} clients in channel '{}': {}",
                sessions.len(),
                channel,
                message
            );
            for client_id in sessions.iter() {
                if let Some(addr) = self.sessions.get(client_id) {
                    let _ = addr.do_send(Message(message.to_owned()));
                }
            }
        }
    }
}

impl Actor for WebSocketServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}


// Messages that can be sent to the WebSocketServer
// Connect - new websocket connection
// Disconnect - close websocket connection
// Subscribe - subscribe websocket connection to a channel

#[derive(Message)]
#[rtype(ClientID)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

impl Handler<Connect> for WebSocketServer {
    type Result = ClientID;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = ClientId(self.counter);
        debug!(LOG, "Registering websocket with id '{}'", id);
        self.counter += 1;
        self.sessions.insert(id, msg.addr);

        id
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: ClientID
}

impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        debug!(LOG, "Unegistering websocket with id '{}'", msg.id);
        self.sessions.remove(&msg.id);
        for (channel_name, channel) in &mut self.channels {
            channel.remove(&msg.id);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    id: ClientID,
    channel: String,
}

impl Handler<Subscribe> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Context<Self>) -> Self::Result {
        debug!(LOG, "Subscribing '{}' to channel '{}'", msg.id, msg.channel);
        self.channels
            .entry(msg.channel.clone())
            .or_insert_with(HashSet::new)
            .insert(msg.id);
        debug!("Channels: {:?}", self.channels);
    }
}
