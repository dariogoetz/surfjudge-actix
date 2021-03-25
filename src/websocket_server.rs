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
                    let _ = addr.do_send(Message {channel: channel.to_owned(), message: message.to_owned()));
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendChannel {
    message: String,
    channel: String,
}

impl Handler<SendChannel> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: SendChannel, _: &mut Context<Self>) -> Self::Result {
        debug!(LOG, "Sending message '{}' to channel '{}'", msg.message, msg.channel);
        self.send_channel(&msg.channel, &msg.message);
    }
}

struct WebSocketSession {
    id: Option<ClientID>,
    server_addr: Addr<WebSocketServer>,
}

impl Actor for WebSocketSession {
    type Context = ws.WebsocketContext<Self>;

    // Method is called on actor start
    // Session is registered to WebSocketServer and receives a ClientID
    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.server_addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    // store client id received from server
                    Ok(res) => act.id = Some(res),
                    // something went wrong when connecting in server
                    _ => ctx.stop(),

                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // disconnect from server
        if let Some(id) = self.id {
            self.server_addr.do_send(Disconnect { id })
        }
        Running::Stop
    }
}

impl Handler<Message> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(json!(msg));
    }
}

// TODO: Handle messages from websocket connection (subscribe)
/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };
    }
