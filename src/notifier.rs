use crate::logging::LOG;

use anyhow::Result;
use slog::{info, warn};
use std::sync::mpsc::{self, Sender};
use std::thread;
use zmq::{Context, PUB};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    ActiveHeats,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotifierMessage {
    channel: Channel,
    message: String,
}

#[derive(Clone)]
pub struct Notifier {
    sender: Sender<NotifierMessage>,
}

impl Notifier {
    pub async fn new(addr: &str) -> Result<Notifier> {
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

        Ok(Notifier {
            sender: server_sender,
        })
    }

    pub async fn send_channel(&self, channel: Channel, message: Value) -> Result<()> {
        let msg = NotifierMessage {
            channel: channel,
            message: message.to_string(),
        };
        Ok(self.sender.send(msg)?)
    }
}
