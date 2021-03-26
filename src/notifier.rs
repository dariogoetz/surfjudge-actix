use crate::logging::LOG;
use crate::websockets::SendChannel;

use std::sync::{Arc, Mutex};
use actix::Recipient;
use anyhow::Result;
use slog::warn;
use std::sync::mpsc::{self, Sender};
use std::thread;
use zmq::{Context, PUB};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    ActiveHeats,
    Results,
    Advancements,
    Participants,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotifierMessage {
    channel: Channel,
    message: String,
}

pub trait Notify {
    fn send_channel(&self, msg: &NotifierMessage) -> Result<()>;
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
    fn send_channel(&self, msg: &NotifierMessage) -> Result<()> {
        let msg = SendChannel {
            channel: msg.channel.clone(),
            message: msg.message.clone()
        };
        self.addr.do_send(msg)?;
        Ok(())
    }
}

pub struct ZMQNotifier {
    addr: Sender<NotifierMessage>
}

impl ZMQNotifier {
    pub fn new(addr: &str) -> Result<Self> {
        let (server_sender, server_receiver) = mpsc::channel::<NotifierMessage>();

        let context = Context::new();
        let publisher = context.socket(PUB).unwrap();
        publisher.connect(addr)?;

        thread::spawn(move || {
            while let Ok(msg) = server_receiver.recv() {
                match publisher.send(&serde_json::to_string(&msg).unwrap(), 0) {
                    Err(e) => warn!(LOG, "Could not send zmq message: {:?}", e),
                    _ => (),
                }
            }
        });
        Ok(Self { addr: server_sender })
    }

}

impl Notify for ZMQNotifier {
    fn send_channel(&self, msg: &NotifierMessage) -> Result<()> {
        self.addr.send(msg.clone())?;
        Ok(())
    }
}


#[derive(Clone)]
pub struct Notifier {
    notifiers: Arc<Mutex<Vec<Box<dyn Notify + Send>>>>,
}

impl Notifier {
    pub fn new() -> Result<Notifier> {
        Ok(Notifier {
            notifiers: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn register(&mut self, notifier: Box<dyn Notify + Send>) -> Result<&mut Self> {
        self.notifiers.lock().unwrap().push(notifier);
        Ok(self)
    }


    pub fn send_channel(&self, channel: Channel, message: Value) -> Result<()> {
        let msg = NotifierMessage {
            channel,
            message: message.to_string(),
        };
        for notifier in self.notifiers.lock().unwrap().iter() {
            notifier.send_channel(&msg)?;
        }
        Ok(())
    }
}
