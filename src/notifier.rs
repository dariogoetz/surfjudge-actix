use crate::logging::LOG;
use crate::websockets::SendChannel;

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

// TODO: vec of impl notifier
#[derive(Clone)]
pub struct Notifier {
    zmq: Option<ZMQNotifier>,
    ws: Option<WSNotifier>,
}

// TODO: trait for notifier
#[derive(Clone)]
pub struct WSNotifier {
    addr: Recipient<SendChannel>,
}

impl WSNotifier {
    pub fn new(addr: Recipient<SendChannel>) -> Result<Self> {
        Ok(Self { addr })
    }

    pub async fn send_channel(&self, msg: &SendChannel) -> Result<()> {
        self.addr.do_send(msg.clone())?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct ZMQNotifier {
    addr: Sender<SendChannel>
}

impl ZMQNotifier {
    pub fn new(addr: &str) -> Result<Self> {
        let (server_sender, server_receiver) = mpsc::channel::<SendChannel>();

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

    pub async fn send_channel(&self, msg: &SendChannel) -> Result<()> {
        self.addr.send(msg.clone())?;
        Ok(())
    }
}

impl Notifier {
    pub fn new() -> Result<Notifier> {
        Ok(Notifier {
            zmq: None,
            ws: None,
        })
    }

    pub fn ws(mut self, notifier: WSNotifier) -> Result<Self> {
        self.ws = Some(notifier);

        Ok(self)
    }

    pub fn zmq(mut self, notifier: ZMQNotifier) -> Result<Self> {
        self.zmq = Some(notifier);

        Ok(self)
    }

    pub async fn send_channel(&self, channel: Channel, message: Value) -> Result<()> {
        let msg = SendChannel {
            channel,
            message: message.to_string(),
        };
        if let Some(sender) = &self.ws {
            sender.send_channel(&msg).await?;
        }
        if let Some(sender) = &self.zmq {
            sender.send_channel(&msg).await?;
        }
        Ok(())
    }
}
