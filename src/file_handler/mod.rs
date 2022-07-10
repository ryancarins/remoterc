use dirs::cache_dir;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use jwalk::WalkDir;
use regex::RegexSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Archive;
use tar::Builder;
use tracing::{event, info, Level};

mod file_handler_error;
use crate::file_handler::file_handler_error::FileHandlerError;

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
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
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

    Ok(cache_dir)
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
    for path in &files {
        info!("Added path to tarball: {}", path.to_string_lossy());
    }

    let encoder = ZlibEncoder::new(dest_file, Compression::best());

    let mut tar_builder = Builder::new(encoder);
    if files.iter().all(|x| x.is_absolute()) {
        for path in files {
            let mut current = File::open(&path).unwrap();
            tar_builder.append_file(
                Path::new("./").join(path.file_name().unwrap()),
                &mut current,
            )?;
        }
    } else {
        for path in files {
            tar_builder.append_path(path)?;
        }
    }

    let zlib = tar_builder.into_inner()?;

    zlib.finish()?;
    Ok(())
}

fn decompress_tarball(src: Vec<u8>, dest: PathBuf) -> Result<(), FileHandlerError> {
    let file_reader = BufReader::new(&*src);
    let decoder = ZlibDecoder::new(file_reader);
    let mut archive = Archive::new(decoder);

    archive.unpack(dest)?;
    Ok(())
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

pub fn process(source: Vec<u8>) -> Result<PathBuf, FileHandlerError> {
    let cache_dir = get_cache_dir()?;
    let build_dir = cache_dir.join(get_timestamp().to_string());
    fs::create_dir(build_dir.clone())?;

    decompress_tarball(source, build_dir.clone())?;

    Ok(build_dir)
}

pub fn create_return_file(executables: Vec<PathBuf>) -> PathBuf {
    let (file, filepath) = create_file().unwrap();

    create_compressed_tarball(&file, executables).expect("Failed to create return file");

    filepath
}

pub fn unzip_executables(archive: Vec<u8>, dest: PathBuf) -> Result<(), FileHandlerError> {
    decompress_tarball(archive, dest)
}
