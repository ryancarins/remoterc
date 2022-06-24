
use std::fs::File;
use std::io;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use flate2::bufread::ZlibDecoder;
use tar::Archive;
use tar::Builder;
use tracing::{event, Level};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use jwalk::WalkDir;
use std::time::{SystemTime, UNIX_EPOCH};
use dirs::cache_dir;
use regex::RegexSet;

#[derive(Debug)]
pub enum FileHandlerError {
    Io(io::Error),
    Jwalk(jwalk::Error),
    Regex(regex::Error)
}

pub fn get_project_file(path: PathBuf, exclusions: &Vec::<String>) -> Result<File, FileHandlerError> {
    let file = create_file()?;

    let files = get_rust_files(path, exclusions)?;
    create_compressed_tarball(&file, files)?;
    
    Ok(file)
}


fn get_timestamp() -> u128 {
    let start = SystemTime::now();
    return start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_millis();
}

fn get_cache_dir() -> Result<PathBuf, FileHandlerError> {
    let cache_dir = cache_dir().unwrap_or_else(|| {
        event!(Level::ERROR, "Critical: Failed to get cachedir. Exiting");
        panic!();
    }).join("rrc");
    
    if !cache_dir.exists() {
        fs::create_dir(&cache_dir).map_err(FileHandlerError::Io)?;
    }
    return Ok(cache_dir);
}

fn create_file() -> Result<File, FileHandlerError> {
    
    let filepath = format!("{}.rrc", get_timestamp());
    
    let cache_dir = get_cache_dir()?;
    
    event!(Level::INFO, "{}", cache_dir.join(&filepath).display());

    let tempfile = File::create(cache_dir.join(filepath)).map_err(FileHandlerError::Io)?;
    
    Ok(tempfile)
}

fn create_compressed_tarball(dest_file: &File, files: Vec<PathBuf>) -> Result<(), FileHandlerError> {
    let encoder = ZlibEncoder::new(dest_file, Compression::best());

    let mut tar_builder = Builder::new(encoder);
    for path in files {
        tar_builder.append_path(path).map_err(FileHandlerError::Io)?;
    }

    let zlib = tar_builder.into_inner().map_err(FileHandlerError::Io)?;

    zlib.finish().map_err(FileHandlerError::Io)?;

    Ok(())
}

fn decompress_tarball(src_file: &File) -> Result<PathBuf, FileHandlerError> {
    let file_reader = BufReader::new(src_file);
    let decoder = ZlibDecoder::new(file_reader);
    let mut archive = Archive::new(decoder);

    let cache_dir = get_cache_dir()?;
    let build_dir = cache_dir.join(get_timestamp().to_string()); 
    fs::create_dir(build_dir.clone()).map_err(FileHandlerError::Io)?;
    
    archive.unpack(build_dir.clone()).map_err(FileHandlerError::Io)?;
    Ok(build_dir)
}

fn get_rust_files(path: PathBuf, exclusions: &Vec<String>) -> Result<Vec<PathBuf>, FileHandlerError> {
    let mut paths : Vec<PathBuf> = Vec::new();
    for file in WalkDir::new(path) {
        paths.push(file.map_err(FileHandlerError::Jwalk)?.path());
    }

    paths = filter_paths(paths, exclusions)?;

    Ok(paths)
}

fn filter_paths(paths: Vec<PathBuf>, exclusions: &Vec::<String>) -> Result<Vec<PathBuf>, FileHandlerError> {
    let regex = RegexSet::new(exclusions).map_err(FileHandlerError::Regex)?;
    let mut filtered_paths = Vec::new();

    for path in paths {
        if !regex.is_match(&(path.to_string_lossy())) {
            filtered_paths.push(path);
            event!(Level::INFO, "Added file: {}", filtered_paths.last().unwrap().display());
        }
    }

    Ok(filtered_paths)
}

pub fn process(file: &File) -> Result<(PathBuf), FileHandlerError> {
    let builddir = decompress_tarball(file)?;
    Ok((builddir))
}