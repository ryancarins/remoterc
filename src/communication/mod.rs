use futures_channel::mpsc::unbounded;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::io;
use std::net::SocketAddr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

type Tx = UnboundedSender<Message>;
type PeerMap = HashMap<SocketAddr, (Option<i32>, Tx)>;

pub async fn create_client_connection(
    rx: UnboundedReceiver<Message>,
    tx: UnboundedSender<Message>,
) {
    let url = url::Url::parse("ws://127.0.0.1:8888").unwrap();

    let ws_stream = match connect_async(url).await {
        Ok((stream, _)) => stream,
        Err(_) => return,
    };

    info!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();
    let send = rx.map(Ok).forward(write);
    let recieve = {
        read.for_each(|message| async {
            let mut tx_inner = tx.clone();
            if message.is_err() {
                error!(
                    "Error recieving message from build server: {}",
                    message.unwrap_err()
                );
                return;
            }

            let send_result = tx_inner.start_send(message.unwrap());
            if send_result.is_err() {
                error!("Error sending message: {}", send_result.unwrap_err());
            }
        })
    };

    pin_mut!(send, recieve);
    future::select(send, recieve).await;
}

pub async fn create_server_listener() -> Result<(), io::Error> {
    let port = 8888;
    let addr_ipv4 = format!("127.0.0.1:{}", port);

    let addr_ipv6 = format!("[::1]:{}", port);

    let state = Arc::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener
    let try_socket_ipv4 = TcpListener::bind(&addr_ipv4).await;
    let listener_ipv4 = try_socket_ipv4;
    let try_socket_ipv6 = TcpListener::bind(&addr_ipv6).await;
    let listener_ipv6 = try_socket_ipv6;

    let mut is_listening_ipv4 = false;
    let mut is_listening_ipv6 = false;

    if listener_ipv4.is_ok() {
        is_listening_ipv4 = true;
        info!("Listening on: {}", addr_ipv4);
    } else {
        warn!("Failed to bind to ipv4: {}", addr_ipv4);
    }

    if listener_ipv6.is_ok() {
        is_listening_ipv6 = true;
        info!("Listening on: {}", addr_ipv6);
    } else {
        warn!("Failed to bind to ipv6: {}", addr_ipv6);
    }

    if !is_listening_ipv4 && !is_listening_ipv6 {
        error!(
            "Could not bind to {} or {}. Websocket connections not possible",
            addr_ipv6, addr_ipv4
        );
    }

    //Handle ipv4 and ipv6 simultaneously and end if ctrl_c is run
    //
    //This looks and is a bit janky. Need to look into a way of specifying
    //a set of tasks for a select to listen to based on a condition instead
    //of using 3 select macros. For now this will work
    loop {
        if is_listening_ipv4 && is_listening_ipv6 {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_server_connection(state.clone(), stream, addr));
                }
                Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_server_connection(state.clone(), stream, addr));
                }
            }
        } else if is_listening_ipv4 {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv4.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_server_connection(state.clone(), stream, addr));
                }
            }
        } else {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    warn!("Ctrl-C received, shutting down");
                    break;
                }
                Ok((stream, addr)) = listener_ipv6.as_ref().unwrap().accept() => {
                    tokio::spawn(handle_server_connection(state.clone(), stream, addr));
                }
            }
        }
    }

    //Close all websocket connection gracefully before exit
    for (_, tx) in (&mut *state.lock().unwrap()).values_mut() {
        let _ = tx.start_send(Message::Close(None));
    }

    Ok(())
}

async fn handle_server_connection(
    peer_map: Arc<Mutex<PeerMap>>,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .unwrap_or_else(|err| {
            error!(
                "Error during the websocket handshake occurred. Err: {}",
                err
            );
            panic!();
        });
    info!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, (None, tx.clone()));
    let (outgoing, incoming) = ws_stream.split();

    let handle_incoming = incoming.try_for_each(|message| {
        match message {
            Message::Text(text) => {
                info!("{text}");
            }
            Message::Binary(binary) => {
                info!("Recieved binary message of size: {}", binary.len());
            }
            _ => {}
        }
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_incoming, receive_from_others);
    future::select(handle_incoming, receive_from_others).await;

    info!("{} disconnected", &addr);
    let mut lock = peer_map.lock().unwrap();

    lock.remove(&addr);
}
