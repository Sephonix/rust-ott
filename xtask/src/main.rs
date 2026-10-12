use nih_plug_xtask::Result;
mod delay;
mod echo;
mod ott;
mod room;
mod time_metadata;
use ott::metadata;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

struct Plugin {
    package: &'static str,
    lv2_package: &'static str,
    name: &'static str,
    metadata_directory: &'static str,
    metadata: fn(&Path) -> std::io::Result<()>,
}

const PLUGINS: &[Plugin] = &[
    Plugin {
        package: "sps-ott",
        lv2_package: "sps-ott-lv2",
        name: "SPS OTT",
        metadata_directory: "plugins/sps-ott/metadata",
        metadata,
    },
    Plugin {
        package: "sps-room",
        lv2_package: "sps-room-lv2",
        name: "SPS Room",
        metadata_directory: "plugins/sps-room/metadata",
        metadata: room::metadata,
    },
    Plugin {
        package: "sps-delay",
        lv2_package: "sps-delay-lv2",
        name: "SPS Delay",
        metadata_directory: "plugins/sps-delay/metadata",
        metadata: delay::metadata,
    },
    Plugin {
        package: "sps-echo",
        lv2_package: "sps-echo-lv2",
        name: "SPS Echo",
        metadata_directory: "plugins/sps-echo/metadata",
        metadata: echo::metadata,
    },
];

fn main() -> Result<()> {
    nih_plug_xtask::chdir_workspace_root()?;
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    if command == "help" || command == "--help" {
        let available = PLUGINS
            .iter()
            .map(|p| p.package)
            .collect::<Vec<_>>()
            .join(", ");
        println!("SPS DSP workspace build tools\n  cargo xtask bundle [plugin] [--release] [--locked]\n  cargo xtask install [plugin] [--release] [--locked]  (Linux, current user)\n  cargo xtask metadata [plugin]\n\nOmit plugin to select all registered plugins. Available: {available}.\nBundles are written to target/bundled/. For cross-platform VST3/CLAP bundles,\nuse cargo xtask bundle sps-ott --release --target=<triple>.");
        return Ok(());
    }
    if !["metadata", "bundle", "install"].contains(&command.as_str()) {
        return Err(std::io::Error::other(format!("unknown command: {command}")).into());
    }
    let mut flags: Vec<String> = args.collect();
    let plugins: Vec<&Plugin> = if flags.first().is_some_and(|s| !s.starts_with('-')) {
        let package = flags.remove(0);
        vec![PLUGINS
            .iter()
            .find(|p| p.package == package)
            .ok_or_else(|| std::io::Error::other(format!("unknown plugin: {package}")))?]
    } else {
        PLUGINS.iter().collect()
    };
    if command == "metadata" {
        if !flags.is_empty() {
            return Err(
                std::io::Error::other("metadata accepts only an optional plugin name").into(),
            );
        }
        for plugin in plugins {
            (plugin.metadata)(Path::new(plugin.metadata_directory))?;
        }
        return Ok(());
    }
    if !flags.iter().any(|s| s == "--locked") {
        flags.push("--locked".into());
    }
    if command == "bundle"
        && flags
            .iter()
            .any(|s| s == "--target" || s.starts_with("--target="))
    {
        for plugin in plugins {
            nih_plug_xtask::main_with_args(
                "cargo xtask",
                [vec![command.clone(), plugin.package.into()], flags.clone()].concat(),
            )?;
        }
        return Ok(());
    }
    if flags
        .iter()
        .any(|s| !["--release", "--locked"].contains(&s.as_str()))
    {
        return Err(std::io::Error::other("supported flags: --release, --locked; cross-compilation: bundle [plugin] --target=<triple> (CLAP/VST3 only)").into());
    }
    if command == "install" && !cfg!(target_os = "linux") {
        return Err(
            std::io::Error::other("user installation currently supports Linux only").into(),
        );
    }
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"));
    let packages: Vec<String> = plugins
        .iter()
        .flat_map(|p| [p.package.to_owned(), p.lv2_package.to_owned()])
        .collect();
    nih_plug_xtask::build(&packages, &flags)?;
    for plugin in plugins {
        nih_plug_xtask::bundle(&target, plugin.package, &flags, false)?;
        let lv2_bundle = format!("{}.lv2", plugin.package);
        if cfg!(target_os = "linux") {
            let bundle = target.join("bundled").join(&lv2_bundle);
            (plugin.metadata)(&bundle)?;
            let profile = if flags.iter().any(|s| s == "--release") {
                "release"
            } else {
                "debug"
            };
            copy_atomic(
                &target
                    .join(profile)
                    .join(format!("lib{}.so", plugin.lv2_package.replace('-', "_"))),
                &bundle.join(format!("{}.so", plugin.package.replace('-', "_"))),
            )?;
        }
        if command == "install" {
            let home = env::var_os("HOME").ok_or_else(|| std::io::Error::other("HOME is unset"))?;
            let home = PathBuf::from(home);
            for (name, directory) in [
                (format!("{}.clap", plugin.name), ".clap"),
                (format!("{}.vst3", plugin.name), ".vst3"),
                (lv2_bundle, ".lv2"),
            ] {
                let destination = home.join(directory).join(&name);
                copy_tree(&target.join("bundled").join(&name), &destination)?;
                println!("Installed {}", destination.display());
            }
        }
        println!(
            "{} bundles: {}",
            plugin.name,
            target.join("bundled").display()
        );
    }
    Ok(())
}
fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        copy_atomic(source, destination)?;
    }
    Ok(())
}

fn copy_atomic(source: &Path, destination: &Path) -> std::io::Result<()> {
    // Never truncate a plugin library that an open DAW may have memory-mapped.
    let temporary = destination.with_extension(format!("{}.tmp", std::process::id()));
    fs::copy(source, &temporary)?;
    if let Err(error) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}
