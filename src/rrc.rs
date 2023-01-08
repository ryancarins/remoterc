use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{self};
use tokio::task;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, error, info};

use remoterc::communication::create_client_connection;
use remoterc::file_handler::get_project_file;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let (mut server_tx, server_rx) = futures_channel::mpsc::unbounded();
    let (client_tx, client_rx) = mpsc::channel();
    let handle = task::spawn(create_client_connection(server_rx, client_tx));

    let exclusions = vec![String::from(r"target/*"), String::from(r"testpath/*")];

    let result = get_project_file(Path::new("./").to_path_buf(), &exclusions);
    let filepath = match result {
        Ok(file) => file,
        Err(error) => {
            error!("Failed to make a project file. Error: {error}");
            std::process::exit(1);
        }
    };

    let mut file = File::open(filepath).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap_or_else(|err| {
        error!(
            "Failed to read archive to send to build server. Error: {}",
            err
        );
        std::process::exit(1);
    });

    while let Ok(message) = client_rx.recv() {
        if message != "listening" {
            error!("Failed to create websocket");
            std::process::exit(1);
        }
        debug!("Got message: {}", message);
    }

    server_tx
        .start_send(Message::Binary(buffer.to_vec()))
        .unwrap();

    handle.await.unwrap_or_else(|err| {
        error!("Error waiting for websocket connection: {}", err);
        std::process::exit(1);
    });
}
