//! Versioned user presets. Call only from UI/background threads, never audio.
use serde::{Deserialize, Serialize};
use sps_dsp::params::Spec;
use std::{
    collections::BTreeMap,
    env,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Preset {
    version: u32,
    plugin: String,
    parameters: BTreeMap<String, f32>,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 80
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || " _-".contains(c))
}
/// Respect Linux XDG user-dirs, including localized Documents directories.
/// An explicitly disabled Documents directory falls back to ~/Documents.
pub fn documents() -> io::Result<PathBuf> {
    let home = PathBuf::from(env::var_os("HOME").ok_or_else(|| invalid("HOME is unset"))?);
    let config = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home.join(".config"));
    if let Ok(text) = fs::read_to_string(config.join("user-dirs.dirs")) {
        for line in text.lines() {
            if let Some(value) = line.trim().strip_prefix("XDG_DOCUMENTS_DIR=") {
                let value = value.trim().trim_matches('"');
                let path = if let Some(rest) = value.strip_prefix("$HOME/") {
                    home.join(rest)
                } else {
                    PathBuf::from(value)
                };
                if path.is_absolute() && path != home {
                    return Ok(path);
                }
            }
        }
    }
    Ok(home.join("Documents"))
}
pub fn directory(plugin_name: &str) -> io::Result<PathBuf> {
    if !valid_name(plugin_name) {
        return Err(invalid("invalid plugin directory"));
    }
    Ok(documents()?.join("SPS").join(plugin_name))
}
pub fn list(directory: &Path) -> io::Result<Vec<String>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|e| e == "json") {
            if let Some(name) = entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| valid_name(s))
            {
                names.push(name.to_owned());
            }
        }
    }
    names.sort();
    Ok(names)
}
pub fn save(
    directory: &Path,
    name: &str,
    plugin: &str,
    specs: &[Spec],
    values: &[f32],
) -> io::Result<()> {
    if !valid_name(name) || specs.len() != values.len() {
        return Err(invalid(
            "use a preset name containing letters, numbers, spaces, _ or -",
        ));
    }
    let preset = Preset {
        version: 1,
        plugin: plugin.to_owned(),
        parameters: specs
            .iter()
            .zip(values)
            .map(|(s, v)| (s.id.to_owned(), s.sanitize(*v)))
            .collect(),
    };
    let bytes = serde_json::to_vec_pretty(&preset)?;
    fs::create_dir_all(directory)?;
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let temp = directory.join(format!(
        ".sps-{}-{}.tmp",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        // Atomic publication without overwriting another preset or following a symlink.
        fs::hard_link(&temp, directory.join(format!("{name}.json")))?;
        fs::File::open(directory)?.sync_all()?;
        Ok(())
    })();
    let _ = fs::remove_file(temp);
    result
}
pub fn load(directory: &Path, name: &str, plugin: &str, specs: &[Spec]) -> io::Result<Vec<f32>> {
    if !valid_name(name) {
        return Err(invalid("invalid preset name"));
    }
    let mut text = String::new();
    fs::File::open(directory.join(format!("{name}.json")))?
        .take(65537)
        .read_to_string(&mut text)?;
    if text.len() > 65536 {
        return Err(invalid("preset exceeds 64 KiB"));
    }
    let preset: Preset = serde_json::from_str(&text)?;
    if preset.version != 1 || preset.plugin != plugin {
        return Err(invalid("preset version or plugin does not match"));
    }
    if preset.parameters.len() != specs.len() {
        return Err(invalid("preset parameter schema does not match"));
    }
    specs
        .iter()
        .map(|s| {
            let v = *preset
                .parameters
                .get(s.id)
                .ok_or_else(|| invalid("missing parameter"))?;
            if !v.is_finite() || v < s.min || v > s.max || (s.discrete && v != v.round()) {
                return Err(invalid("parameter outside supported range"));
            }
            Ok(v)
        })
        .collect()
}
