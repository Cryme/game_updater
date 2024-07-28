mod packet_handler;
use crate::admin_panel::packet_handler::HandleClientPacket;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use futures_util::StreamExt;
use shared::admin_panel::{ClientPacket, ServerPacket};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use tracing::{debug, error};

pub async fn admin_socket_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ws = ws.max_message_size(500 * 1024 * 1024);

    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    debug!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("Pinged {who}...");
    } else {
        error!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // this will likely be the Pong for our Ping or a hello message from client.
    // waiting for message from a client will block this task, but will not block other client's
    // connections.
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(msg) => match process_message(msg, who).await {
                ControlFlow::Continue(p) => {
                    if let Some(packet) = p {
                        let data = packet.to_bin();

                        if let Err(e) = data {
                            error!("Error encoding packet: {e}");
                            continue;
                        }

                        if let Err(e) = socket.send(Message::Binary(data.unwrap())).await {
                            debug!("Can't send: {e}");
                            return;
                        }
                    };
                }
                ControlFlow::Break(_) => {
                    return;
                }
            },

            Err(e) => {
                debug!("Can't receive {e}");
                return;
            }
        }
    }
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
async fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<ServerPacket>> {
    match msg {
        Message::Text(t) => {
            debug!(">>> {who} sent text {t}");
        }
        Message::Binary(data) => {
            debug!(">>> New packet!");

            match ClientPacket::from_bin(&data) {
                Ok(packet) => {
                    return ControlFlow::Continue(packet.handle().await);
                }
                Err(e) => {
                    error!(">>> Deserialize: {e}");
                }
            }
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                debug!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                debug!(">>> {who} somehow sent close message without CloseFrame");
            }

            return ControlFlow::Break(());
        }
        Message::Pong(v) => {
            debug!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            debug!(">>> {who} sent ping with {v:?}");
        }
    }

    ControlFlow::Continue(None)
}
