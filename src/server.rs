use std::fs::File;
use tracing_subscriber;

mod file_handler;
mod client_communication;

fn main() {
    let subscriber = tracing_subscriber::fmt()
    .compact()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_target(false)
    .finish();
    
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let file = File::open("/home/ryan/.cache/rrc/1656067975961.rrc").expect("Lmao");
    file_handler::process(&file).expect("lmao");
}