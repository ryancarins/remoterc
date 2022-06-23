use std::io::Read;
use std::path::{Path};
use std::fs;
use tracing_subscriber;

mod file_handler;

fn main() {
    let subscriber = tracing_subscriber::fmt()
    .compact()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_target(false)
    .finish();
    
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let file = file_handler::get_temp_file(Path::new("./testpath").to_path_buf());
}
