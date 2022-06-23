use std::fs::File;
use std::fs;
use std::path::PathBuf;
use tar::Builder;
use tracing::{event, Level};
use flate2::Compression;
use flate2::write::GzEncoder;
use jwalk::WalkDir;
use std::time::{SystemTime, UNIX_EPOCH};
use dirs::cache_dir;

pub fn get_temp_file(path: PathBuf) -> File {
    let file = create_file();

    let files = get_rust_files(path);
    create_compressed_tarball(&file, files);
    
    return file;
}

pub fn create_file() -> File {
    let start = SystemTime::now();
    let timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_millis();
    
    let filepath = format!("{}.tar.gz", timestamp);
    
    let cache_dir = cache_dir().unwrap_or_else(|| {
        event!(Level::ERROR, "Failed to get cachedir.");
        panic!();
    }).join("rrc");
    event!(Level::INFO, "{}", cache_dir.join("rrc").join(&filepath).display());

    if(!cache_dir.exists()) {
        fs::create_dir(&cache_dir.to_str().unwrap()).unwrap_or_else(|err| {
            event!(Level::ERROR, "Failed to create cache directory. Error: {err}");
            panic!();
        });
    }

    let tempfile = File::create(cache_dir.join(filepath)).unwrap_or_else(|err| {
        event!(Level::ERROR, "Could not create filepath. Error: {err}");
        panic!();
    });
    
    return tempfile;
}

pub fn create_compressed_tarball(dest_file: &File, files: Vec<PathBuf>) {
    let encoder = GzEncoder::new(dest_file, Compression::best());

    let mut tar_builder = Builder::new(encoder);
    for path in files {
        tar_builder.append_path(path).unwrap_or_else(|err| {
            event!(Level::ERROR, "Adding file to tarball failed. Error: {err}");
            panic!();
        });
    }

    let zlib = tar_builder.into_inner().unwrap_or_else(|err| {
        event!(Level::ERROR, "Tarball creation failed. Error: {err}");
        panic!();
    });

    zlib.finish().unwrap_or_else(|err| {
        event!(Level::ERROR, "Zlib compressed tarball creation failed. Error: {err}");
        panic!();
    });
}

pub fn get_rust_files(path: PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(path) {
        match entry {
            Ok(file) => {
                paths.push(file.path().to_path_buf());
                let path_string = paths.last().unwrap().display();
                event!(Level::INFO, "Added file: {path_string}");
            },
            Err(err) => {
                event!(Level::WARN, "Failed to read file. Error: {err}");
            }
        }
    }
    return paths;
}