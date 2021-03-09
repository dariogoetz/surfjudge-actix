use crate::logging::LOG;

use zmq::{PUB, Context};
use std::sync::mpsc::{self, Sender};
use std::thread;
use anyhow::Result;
use slog::{warn, info};

use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct ZMQMessage {
    pub channel: String,
    pub message: String,
}

#[derive(Clone)]
pub struct Notifier {
    sender: Sender<ZMQMessage>
}

impl Notifier {
    pub fn new(addr: &str) -> Notifier{
        let (server_sender, server_receiver) = mpsc::channel::<ZMQMessage>();
        
        let context = Context::new();
        let publisher = context.socket(PUB).unwrap();
        publisher.connect(addr).expect("Could not connect to zmq publisher");
    
        thread::spawn(move || {
            while let Ok(msg) = server_receiver.recv() {
                info!(LOG, "Sending ZMQ message to websocket server");

                match publisher.send(&serde_json::to_string(&msg).unwrap(), 0) {
                    Err(e) => warn!(LOG, "Could not send zmq message: {:?}", e),
                    _ => ()
                }
            }
        });
    
        Notifier { sender: server_sender }
    }

    pub fn send(&self, msg: ZMQMessage) -> Result<()> {
        info!(LOG, "Sending message to ZMQ notifier");
        Ok(self.sender.send(msg)?)
    }
}

