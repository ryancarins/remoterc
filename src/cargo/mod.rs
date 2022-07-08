use cargo_toml::Manifest;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info};

pub fn cargo_build(builddir: PathBuf, toolchain: String, release: bool, exe: bool) -> Vec<PathBuf> {
    let cargo_toml = Manifest::from_path(builddir.join("Cargo.toml")).expect("No Cargo toml");
    let mut args = vec!["build".to_string()];

    target_add(toolchain.clone());

    args.push("--target".to_string());
    args.push(toolchain);
    if release {
        args.push("--release".to_string());
    }

    let _command = Command::new("cargo")
        .current_dir(&builddir)
        .args(args)
        .output()
        .unwrap_or_else(|err| {
            error!("Failed to run cargo. Error: {err}");
            panic!();
        });

    info!(
        "Cargo succesfully finished building: {}",
        builddir.to_string_lossy()
    );

    let mut executable_path = builddir;
    executable_path.push("target");
    if release {
        executable_path.push("release");
    } else {
        executable_path.push("debug");
    }

    let mut executable_paths = Vec::new();

    for bin in cargo_toml.bin {
        if exe {
            executable_paths.push(executable_path.clone().join(bin.name.unwrap() + ".exe"));
        } else {
            executable_paths.push(executable_path.clone().join(bin.name.unwrap()));
        }
    }

    executable_paths
}

fn target_add(toolchain: String) {
    let mut args = vec![
        "target".to_string(),
        "add".to_string(),
        "x86_64-pc-windows-gnu".to_string(),
    ];
    args.push(toolchain);
    let _command = Command::new("rustup")
        .args(args)
        .output()
        .unwrap_or_else(|err| {
            error!("Failed to add toolchain. Error: {err}");
            panic!();
        });
}
