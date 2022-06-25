use remoterc::communication::create_server_listener;
use tokio::task;

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

    let handle = task::spawn(create_server_listener());
    handle.await.unwrap().unwrap();
}
