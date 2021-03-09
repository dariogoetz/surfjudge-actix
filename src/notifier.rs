use crate::logging::LOG;

use anyhow::Result;
use slog::{info, warn};
use std::sync::mpsc::{self, Sender};
use std::thread;
use zmq::{Context, PUB};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct ZMQMessage {
    pub channel: String,
    pub message: String,
}

#[derive(Clone)]
pub struct Notifier {
    sender: Sender<ZMQMessage>,
}

impl Notifier {
    pub async fn new(addr: &str) -> Result<Notifier> {
        let (server_sender, server_receiver) = mpsc::channel::<ZMQMessage>();

        let context = Context::new();
        let publisher = context.socket(PUB).unwrap();
        publisher.connect(addr)?;

        thread::spawn(move || {
            while let Ok(msg) = server_receiver.recv() {
                info!(LOG, "Sending ZMQ message to websocket server");

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

    pub async fn send(&self, msg: ZMQMessage) -> Result<()> {
        Ok(self.sender.send(msg)?)
    }
}
