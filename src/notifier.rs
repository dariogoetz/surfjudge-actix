use crate::logging::LOG;
use crate::websockets::SendChannel;

use std::sync::RwLock;
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
struct NotifierMessage {
    channel: Channel,
    message: String,
}

trait Notify {
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

#[derive(Clone)]
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

        Ok(Self { addr: server_sender})
    }

}

impl Notify for ZMQNotifier {

    fn send_channel(&self, msg: &NotifierMessage) -> Result<()> {
        self.addr.send(msg.clone())?;
        Ok(())
    }
}



// TODO: vec of impl notifier
#[derive(Clone)]
pub struct Notifier {
    notifiers: RwLock<Vec<Box<dyn Notify + Sync + Send>>>,
    zmq: Option<ZMQNotifier>,
    ws: Option<WSNotifier>,
}

impl Notifier {
    pub fn new() -> Result<Notifier> {
        Ok(Notifier {
            notifiers: Arc::new(Vec::new()),
            zmq: None,
            ws: None,
        })
    }

    pub fn register(mut self, notifier: Box<dyn Notify + Sync + Send>) -> Result<Self> {
        self.notifiers.write().push(notifier);

        Ok(self)
    }

    pub fn ws(mut self, notifier: WSNotifier) -> Result<Self> {
        self.ws = Some(notifier);

        Ok(self)
    }

    pub fn zmq(mut self, notifier: ZMQNotifier) -> Result<Self> {
        self.zmq = Some(notifier);

        Ok(self)
    }

    pub fn send_channel(&self, channel: Channel, message: Value) -> Result<()> {
        let msg = NotifierMessage {
            channel,
            message: message.to_string(),
        };
        for notifier in self.notifiers.read().iter() {
            notifier.send_channel(&msg)?;
        }
        if let Some(sender) = &self.ws {
            sender.send_channel(&msg)?;
        }
        if let Some(sender) = &self.zmq {
            sender.send_channel(&msg)?;
        }
        Ok(())
    }
}
