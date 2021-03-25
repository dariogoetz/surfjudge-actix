use crate::logging::LOG;
use crate::websocket_server::SendChannel;

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

#[derive(Clone)]
pub struct Notifier {
    zmq: Option<Sender<SendChannel>>,
    ws: Option<Recipient<SendChannel>>,
}

impl Notifier {
    pub fn new() -> Result<Notifier> {
        Ok(Notifier {
            zmq: None,
            ws: None,
        })
    }

    pub fn ws(mut self, addr: Recipient<SendChannel>) -> Result<Self> {
        self.ws = Some(addr);

        Ok(self)
    }

    pub fn zmq(mut self, addr: &str) -> Result<Notifier> {
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

        self.zmq = Some(server_sender);

        Ok(self)
    }

    pub async fn send_channel(&self, channel: Channel, message: Value) -> Result<()> {
        let msg = SendChannel {
            channel: channel.clone(),
            message: message.to_string(),
        };
        if let Some(sender) = &self.ws {
            sender.do_send(msg.clone())?;
        }
        if let Some(sender) = &self.zmq {
            sender.send(msg.clone())?;
        }
        Ok(())
    }
}
