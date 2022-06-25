use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info};

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
                return;
            }
        })
    };

    pin_mut!(send, recieve);
    future::select(send, recieve).await;
}
