use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::task;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info};

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
    let (client_tx, client_rx) = futures_channel::mpsc::unbounded();
    let handle = task::spawn(create_client_connection(server_rx, client_tx.clone()));

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
    file.read_to_end(&mut buffer).unwrap();

    info!("{}", buffer.len());

    server_tx.start_send(Message::Binary(buffer)).unwrap();
    handle.await.unwrap_or_else(|err| {
        error!("Error waiting for websocket connection: {}", err);
        std::process::exit(1);
    });
}
