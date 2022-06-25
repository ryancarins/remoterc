use dirs::cache_dir;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use jwalk::WalkDir;
use regex::RegexSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Archive;
use tar::Builder;
use tracing::{event, Level};

mod file_handler_error;
use crate::file_handler::file_handler_error::file_handler_error::FileHandlerError;

pub fn get_project_file(
    path: PathBuf,
    exclusions: &Vec<String>,
) -> Result<PathBuf, FileHandlerError> {
    let (file, filepath) = create_file()?;

    let files = get_rust_files(path, exclusions)?;
    create_compressed_tarball(&file, files)?;

    Ok(filepath)
}

fn get_timestamp() -> u128 {
    let start = SystemTime::now();
    return start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
}

fn get_cache_dir() -> Result<PathBuf, FileHandlerError> {
    let cache_dir = cache_dir()
        .unwrap_or_else(|| {
            event!(Level::ERROR, "Critical: Failed to get cachedir. Exiting");
            panic!();
        })
        .join("rrc");
    event!(Level::INFO, "{}", cache_dir.to_string_lossy());
    if !cache_dir.exists() {
        fs::create_dir(&cache_dir)?;
    }

    return Ok(cache_dir);
}

fn create_file() -> Result<(File, PathBuf), FileHandlerError> {
    let filename = format!("{}.rrc", get_timestamp());

    let cache_dir = get_cache_dir()?;
    let file_path = cache_dir.join(&filename);
    event!(
        Level::INFO,
        "Creating project file in: {}",
        file_path.display()
    );

    let tempfile = File::options()
        .create_new(true)
        .write(true)
        .open(&file_path)?;

    Ok((tempfile, file_path))
}

fn create_compressed_tarball(
    dest_file: &File,
    files: Vec<PathBuf>,
) -> Result<(), FileHandlerError> {
    let encoder = ZlibEncoder::new(dest_file, Compression::best());

    let mut tar_builder = Builder::new(encoder);
    for path in files {
        tar_builder.append_path(path)?;
    }

    let zlib = tar_builder.into_inner()?;

    zlib.finish()?;
    Ok(())
}

fn decompress_tarball(src_file: &File) -> Result<PathBuf, FileHandlerError> {
    let file_reader = BufReader::new(src_file);
    let decoder = ZlibDecoder::new(file_reader);
    let mut archive = Archive::new(decoder);

    let cache_dir = get_cache_dir()?;
    let build_dir = cache_dir.join(get_timestamp().to_string());
    fs::create_dir(build_dir.clone())?;

    archive.unpack(build_dir.clone())?;
    Ok(build_dir)
}

fn get_rust_files(
    path: PathBuf,
    exclusions: &Vec<String>,
) -> Result<Vec<PathBuf>, FileHandlerError> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for file in WalkDir::new(path) {
        paths.push(file?.path());
    }

    paths = filter_paths(paths, exclusions)?;

    Ok(paths)
}

fn filter_paths(
    paths: Vec<PathBuf>,
    exclusions: &Vec<String>,
) -> Result<Vec<PathBuf>, FileHandlerError> {
    let regex = RegexSet::new(exclusions)?;
    let mut filtered_paths = Vec::new();

    for path in paths {
        if !regex.is_match(&(path.to_string_lossy())) {
            filtered_paths.push(path);
            event!(
                Level::INFO,
                "Added file: {}",
                filtered_paths.last().unwrap().display()
            );
        }
    }

    Ok(filtered_paths)
}

pub fn process(file: &File) -> Result<PathBuf, FileHandlerError> {
    let builddir = decompress_tarball(file)?;
    Ok(builddir)
}
