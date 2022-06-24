use std::path::{Path};
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

    let mut exclusions = Vec::new();
    exclusions.push(String::from(r"target/*"));
    exclusions.push(String::from(r"testpath/*"));
    let file = file_handler::get_project_file(Path::new("./").to_path_buf(), &exclusions);


}
