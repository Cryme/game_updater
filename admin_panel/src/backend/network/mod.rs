use futures::channel::mpsc::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use log::{log, Level};
use wasm_bindgen_futures::spawn_local;

use crate::WS_SERVER;
use reqwasm::websocket::{futures::WebSocket, Message};
use shared::admin_panel::{ClientPacket, ServerPacket};

pub struct Network {
    to_client: Option<Sender<ClientPacket>>,
    from_server: std::sync::mpsc::Sender<ServerPacket>,
}

impl Network {
    pub(super) fn send_packet(&self, packet: ClientPacket) {
        log!(Level::Debug, "Uploading --1");
        if let Some(mut v) = self.to_client.clone() {
            log!(Level::Debug, "Uploading --2");
            spawn_local(async move {
                log!(Level::Debug, "Uploading --3");
                let _ = v.send(packet).await;

                log!(Level::Debug, "Uploading --4");
            });
        }
    }

    pub fn init(tx: std::sync::mpsc::Sender<ServerPacket>) -> Self {
        Self {
            to_client: None,
            from_server: tx,
        }
    }

    pub fn run(&mut self) {
        let (tx, rx) = futures::channel::mpsc::channel::<ClientPacket>(100);

        self.to_client = Some(tx);

        let tx = self.from_server.clone();

        spawn_local(async move {
            create_connection(tx, rx).await;
        });
    }
}

async fn try_connect() -> WebSocket {
    loop {
        match WebSocket::open(WS_SERVER) {
            Ok(socket) => {
                log!(Level::Debug, "Handshake has been completed");

                return socket;
            }
            Err(e) => {
                log!(Level::Debug, "WebSocket handshake failed with {e}!");
            }
        };

        TimeoutFuture::new(1_000).await;
    }
}

async fn create_connection(
    tx: std::sync::mpsc::Sender<ServerPacket>,
    mut rx: Receiver<ClientPacket>,
) {
    let ws = try_connect().await;

    let (mut sender, mut receiver) = ws.split();
    //spawn an async sender to push some more messages into the server
    spawn_local(async move {
        while let Some(packet) = rx.next().await {
            log!(Level::Debug, "Encoding packet");

            let start = chrono::Local::now();

            match packet.to_bin() {
                Ok(data) => {
                    log!(
                        Level::Debug,
                        "Encoded in {} milliseconds",
                        chrono::Local::now()
                            .signed_duration_since(start)
                            .num_milliseconds()
                    );

                    let start = chrono::Local::now();

                    match sender.send(Message::Bytes(data)).await {
                        Ok(_) => {
                            log!(
                                Level::Debug,
                                "Ws message sent in {} milliseconds",
                                chrono::Local::now()
                                    .signed_duration_since(start)
                                    .num_milliseconds()
                            );
                        }
                        Err(e) => {
                            log!(Level::Error, "WebSocket send failed with error: {e}!");

                            return;
                        }
                    }
                }
                Err(e) => {
                    log!(Level::Error, "Packet encode error: {e}!");
                }
            }
        }
    });

    spawn_local(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(t)) => {
                    println!(">>> got str: {t:?}");
                }
                Ok(Message::Bytes(b)) => {
                    if let Ok(packet) = ServerPacket::from_bin(&b) {
                        tx.send(packet).unwrap();
                    }
                }
                Err(_) => {}
            }
        }
    });
}
