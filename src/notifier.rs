use crate::logging::LOG;
use crate::websockets::SendChannel;

use actix::Recipient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slog::{debug, warn};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;
use zmq::{Context, PUB};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    ActiveHeats,
    Results,
    Advancements,
    Participants,
}

// Message type sent to notifiers
// sent_by contains all notifier-servers that have sent this message already
// this prevents endless ping-pong between notifiers
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotifierMessage {
    channel: Channel,
    message: String,
    sent_by: Vec<Uuid>,
}

// Old style ZMQ message type without sent_by
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotifierMessageOldStyle {
    channel: Channel,
    message: String,
}

pub trait Notify {
    fn send(&self, msg: &NotifierMessage) -> Result<()>;
}

#[derive(Clone)]
pub struct WSNotifier {
    addr: Recipient<SendChannel>,
}

impl WSNotifier {
    pub fn new(addr: Recipient<SendChannel>) -> Result<Self> {
        Ok(Self { addr })
    }
}

impl Notify for WSNotifier {
    fn send(&self, msg: &NotifierMessage) -> Result<()> {
        let msg = SendChannel {
            channel: msg.channel.clone(),
            message: msg.message.clone(),
        };
        self.addr.do_send(msg)?;
        Ok(())
    }
}

pub struct ZMQNotifier {
    addr: Sender<NotifierMessage>,
}

impl ZMQNotifier {
    pub fn new(addr: &str) -> Result<Self> {
        let (server_sender, server_receiver) = mpsc::channel::<NotifierMessage>();

        let context = Context::new();
        let publisher = context.socket(PUB).unwrap();
        publisher.connect(addr)?;

        thread::spawn(move || {
            debug!(LOG, "Started ZMQ sender thread");

            while let Ok(msg) = server_receiver.recv() {
                let msg = serde_json::to_string(&msg).unwrap();
                debug!(LOG, "Sending ZMQ message: {}", msg);
                match publisher.send(&msg, 0) {
                    Err(e) => warn!(LOG, "Could not send zmq message: {:?}", e),
                    _ => (),
                }
            }
        });
        Ok(Self {
            addr: server_sender,
        })
    }
}

impl Notify for ZMQNotifier {
    fn send(&self, msg: &NotifierMessage) -> Result<()> {
        self.addr.send(msg.clone())?;
        Ok(())
    }
}

pub struct ZMQReceiver {
    notifier: Notifier,
    address: String,
}

impl ZMQReceiver {
    pub fn new(addr: &str, notifier: &Notifier) -> Result<Self> {
        let address = addr.to_string();
        let notifier = notifier.clone();

        Ok(ZMQReceiver { address, notifier })
    }

    pub fn start(&self) -> Result<()> {
        let addr = self.address.clone();
        let notifier = self.notifier.clone();

        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        subscriber.set_subscribe(b"").unwrap();
        subscriber.bind(&addr).expect(&format!(
            "Could not bind address {} for ZMQ receiver",
            &addr
        ));

        thread::spawn(move || {
            debug!(LOG, "Started ZMQ listener thread");

            loop {
                let msg = match subscriber.recv_msg(0) {
                    Ok(x) => x,
                    Err(_err) => {
                        warn!(LOG, "Error while reading zmq message");
                        continue;
                    }
                };
                let msg = match std::str::from_utf8(&msg) {
                    Ok(x) => x,
                    Err(_err) => {
                        warn!(LOG, "Error while parsing zmq message to utf-8");
                        continue;
                    }
                };
                let notifier_msg: NotifierMessage = match serde_json::from_str(&msg) {
                    Ok(x) => x,
                    Err(_err) => match serde_json::from_str::<NotifierMessageOldStyle>(&msg) {
                        Ok(x) => NotifierMessage {
                            channel: x.channel,
                            message: x.message,
                            sent_by: Vec::new(),
                        },
                        Err(_err) => {
                            warn!(LOG, "Error parsing message to json: {}", msg);
                            continue;
                        }
                    },
                };
                debug!(LOG, "Received ZMQ Message '{}'", msg);
                notifier.forward(notifier_msg).unwrap_or_else(|_error| {
                    warn!(LOG, "Could not forward zmq message '{}' to server", msg);
                });
            }
        });
        Ok(())
    }
}

#[derive(Clone)]
pub struct Notifier {
    notifiers: Arc<Mutex<Vec<Box<dyn Notify + Send>>>>,
    id: Uuid,
}

impl Notifier {
    pub fn new() -> Result<Notifier> {
        Ok(Notifier {
            notifiers: Arc::new(Mutex::new(Vec::new())),
            id: Uuid::new_v4(),
        })
    }

    pub fn register(&mut self, notifier: Box<dyn Notify + Send>) -> Result<&mut Self> {
        self.notifiers.lock().unwrap().push(notifier);
        Ok(self)
    }

    pub fn forward(&self, mut msg: NotifierMessage) -> Result<()> {
        if msg.sent_by.iter().any(|id| *id == self.id) {
            debug!(LOG, "Not forwarding message already sent before");
        } else {
            msg.sent_by.push(self.id.clone());
            for notifier in self.notifiers.lock().unwrap().iter() {
                notifier.send(&msg)?;
            }
        }
        Ok(())
    }

    pub fn send(&self, channel: Channel, message: Value) -> Result<()> {
        let msg = NotifierMessage {
            channel,
            message: message.to_string(),
            sent_by: vec![self.id.clone()],
        };
        for notifier in self.notifiers.lock().unwrap().iter() {
            notifier.send(&msg)?;
        }
        Ok(())
    }
}
