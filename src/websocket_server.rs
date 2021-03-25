use crate::logging::LOG;
use crate::notifier::Channel;

use std::{fmt, collections::{HashMap, HashSet}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use slog::{debug, info, warn};

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;


/// Entry point for our websocket route
pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<WebSocketServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WebSocketSession {
            id: None,
            server_addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}


#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug, Hash, Copy)]
pub struct ClientID(usize);

impl fmt::Display for ClientID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WSActionMessage {
    pub action: String,
    pub channel: Channel,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Serialize, Deserialize, Debug)]
pub struct WSMessage {
    pub channel: Channel,
    pub message: String,
}

pub struct WebSocketServer {
    // Recipient is an actix actor
    sessions: HashMap<ClientID, Recipient<WSMessage>>,
    channels: HashMap<Channel, HashSet<ClientID>>,
    counter: usize,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            channels: HashMap::new(),
            counter: 0,
        }
    }

    fn send_channel(&self, channel: &Channel, message: &str) {
        if let Some(sessions) = self.channels.get(channel) {
            let message = json!({"channel": channel, "message": message});
            info!(
                LOG,
                "Sending message to {} clients in channel '{:?}': {}",
                sessions.len(),
                channel,
                message
            );
            for client_id in sessions.iter() {
                if let Some(addr) = self.sessions.get(client_id) {
                    let _ = addr.do_send(WSMessage { channel: channel.to_owned(), message: message.to_string() });
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
// WSMessage - send a websocket message to all clients in a channel

#[derive(Message)]
#[rtype(result = "ClientID")]
pub struct Connect {
    pub addr: Recipient<WSMessage>,
}

impl Handler<Connect> for WebSocketServer {
    type Result = MessageResult<Connect>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = ClientID(self.counter);
        debug!(LOG, "Registering websocket with id '{}'", id);
        self.counter += 1;
        self.sessions.insert(id, msg.addr);

        MessageResult(id)
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
        for (_, channel) in &mut self.channels {
            channel.remove(&msg.id);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    id: ClientID,
    channel: Channel,
}

impl Handler<Subscribe> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Context<Self>) -> Self::Result {
        debug!(LOG, "Subscribing '{}' to channel '{:?}'", msg.id, msg.channel);
        self.channels
            .entry(msg.channel.to_owned())
            .or_insert_with(HashSet::new)
            .insert(msg.id);
        debug!(LOG, "Channels: {:?}", self.channels);
    }
}

#[derive(Serialize, Message, Clone)]
#[rtype(result = "()")]
pub struct SendChannel {
    pub message: String,
    pub channel: Channel,
}

impl Handler<SendChannel> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: SendChannel, _: &mut Context<Self>) -> Self::Result {
        debug!(LOG, "Sending message '{}' to channel '{:?}'", msg.message, msg.channel);
        self.send_channel(&msg.channel, &msg.message);
    }
}

struct WebSocketSession {
    id: Option<ClientID>,
    server_addr: Addr<WebSocketServer>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

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

impl Handler<WSMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: WSMessage, ctx: &mut Self::Context) {
        ctx.text(json!(msg).to_string());
    }
}

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

        match msg {
            ws::Message::Ping(msg) => {
                ctx.pong(&msg);
            }
            ws::Message::Pong(msg) => (),
            ws::Message::Text(msg) => {
                let msg: WSActionMessage = match serde_json::from_str(&msg) {
                    Ok(msg) => msg,
                    Err(_err) => {
                        warn!(LOG,
                            "Error parsing websocket message '{}' to json. Unknown channel?",
                            msg
                        );
                        return;
                    }
                };
                debug!(LOG, "Dispatching message from WebSocket: {:?}", msg);
                if msg.action == "subscribe" {
                    if let Some(id) = self.id {
                        self.server_addr.do_send(Subscribe { id, channel: msg.channel });
                    }
                } else {
                    warn!(LOG, "Unknown action: '{}'", msg.action);
                }
            }
            ws::Message::Binary(_) => warn!(LOG, "Unexpected binary websocket message!"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
            _ => (),
        }
    }
}
