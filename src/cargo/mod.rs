use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info};

pub fn cargo_build(builddir: PathBuf, release: bool) {
    let mut args = vec!["build".to_string()];

    if release {
        args.push("--release".to_string());
    }

    let mut base_command = Command::new("cargo")
        .current_dir(&builddir)
        .args(args)
        .output()
        .unwrap_or_else(|err| {
            error!("Failed to run cargo. Error: {err}");
            panic!();
        });

    info!(
        "Cargo finished succesfully building: {}",
        builddir.to_string_lossy()
    );
}
