use crate::{map_nes_err, nes_err, serialization::Savable, NesResult};
use pix_engine::image::Image;
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

const CONFIG_DIR: &str = ".rustynes";
const RECENTS: &str = "recents";

/// Searches for valid NES rom files ending in `.nes` at the given path
pub fn find_roms<P: AsRef<Path>>(path: &P) -> NesResult<Vec<PathBuf>> {
    let mut roms: Vec<PathBuf> = Vec::new();
    let path = path.as_ref();
    if path.is_dir() {
        path.read_dir()
            .map_err(|e| map_nes_err!("unable to read directory {:?}: {}", path, e))?
            .filter_map(|f| f.ok())
            .filter(|f| f.path().extension() == Some(OsStr::new("nes")))
            .for_each(|f| roms.push(f.path()));
    } else if path.is_file() {
        roms.push(path.to_path_buf());
    } else {
        nes_err!("invalid path: {:?}", path)?;
    }
    Ok(roms)
}

/// Returns a list of recently played roms, ordered by last played
pub fn get_recent_roms() -> NesResult<Vec<(PathBuf, Image)>> {
    // Load recents file to get a list of recent rom paths
    let recents_dir = config_dir().join(RECENTS);
    let recents_path = recents_dir.join(RECENTS).with_extension("dat");
    let mut recents: HashSet<String> = HashSet::new();
    if recents_path.exists() {
        let recents_file = File::open(&recents_path).map_err(|e| {
            map_nes_err!("failed to open recent games file {:?}: {}", recents_path, e)
        })?;
        let mut recents_file = BufReader::new(recents_file);
        recents.load(&mut recents_file)?;
    }

    // Load rom path and image into a list
    let mut results: Vec<(PathBuf, Image)> = Vec::new();
    for rom in recents {
        let rom_path = PathBuf::from(rom);
        let rom_file = rom_path.file_name().expect("valid rom");
        let image_file = recents_dir.join(rom_file).with_extension("png");
        let image = Image::from_file(image_file.to_str().unwrap())?;
        results.push((rom_path, image));
    }
    Ok(results)
}

/// Returns a list of recently played roms, ordered by last played
pub fn add_recent_rom<P: AsRef<Path>>(rom: &P, image: Image) -> NesResult<()> {
    // Ensure recents dir exists
    let recents_dir = config_dir().join(RECENTS);
    if !recents_dir.exists() {
        fs::create_dir_all(&recents_dir).map_err(|e| {
            map_nes_err!(
                "failed to create recent games directory {:?}: {}",
                recents_dir,
                e
            )
        })?;
    }

    // Save rom screenshot
    let rom_file = rom.as_ref().file_name().expect("valid rom path");
    let image_file = recents_dir.join(rom_file).with_extension("png");
    image.save_to_file(image_file.to_str().unwrap())?;

    // If recent games exist, load them
    let mut recents: HashSet<String> = HashSet::new();
    let recents_path = recents_dir.join(RECENTS).with_extension("dat");
    if recents_path.exists() {
        let recents_file = File::open(&recents_path).map_err(|e| {
            map_nes_err!("failed to open recent games file {:?}: {}", recents_path, e)
        })?;
        let mut recents_file = BufReader::new(recents_file);
        recents.load(&mut recents_file)?;
    }

    // Save out recent games list
    recents.insert(rom.as_ref().to_string_lossy().to_string());
    let recents_file = File::create(&recents_path).map_err(|e| {
        map_nes_err!(
            "failed to write recent games file {:?}: {}",
            recents_path,
            e
        )
    })?;
    let mut recents_file = BufWriter::new(recents_file);
    recents.save(&mut recents_file)?;
    Ok(())
}

/// Returns valid directories at the given path
pub fn list_dirs<P: AsRef<Path>>(path: &P) -> NesResult<Vec<PathBuf>> {
    let path = path.as_ref();
    let mut paths: Vec<PathBuf> = Vec::new();
    path.read_dir()
        .map_err(|e| map_nes_err!("unable to read directory {:?}: {}", path, e))?
        .filter_map(|result| result.ok())
        .map(|direntry| direntry.path())
        .filter(|path| path.is_dir())
        .for_each(|s| paths.push(s));
    Ok(paths)
}

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("./"))
        .join(CONFIG_DIR)
}

/// Returns the path where battery-backed Save RAM files are stored
///
/// # Arguments
///
/// * `path` - An object that implements AsRef<Path> that holds the path to the currently
/// running ROM
///
/// # Errors
///
/// Errors if path is not a valid path
pub fn sram_path<P: AsRef<Path>>(path: &P) -> NesResult<PathBuf> {
    let save_name = path.as_ref().file_stem().and_then(|s| s.to_str()).unwrap();
    let path = config_dir()
        .join("sram")
        .join(save_name)
        .with_extension("sram");
    Ok(path)
}

/// Returns the path where Save states are stored
///
/// # Arguments
///
/// * `path` - An object that implements AsRef<Path> that holds the path to the currently
/// running ROM
///
/// # Errors
///
/// Errors if path is not a valid path
pub fn save_path<P: AsRef<Path>>(path: &P, slot: u8) -> NesResult<PathBuf> {
    if let Some(save_name) = path.as_ref().file_stem().and_then(|s| s.to_str()) {
        let path = config_dir()
            .join("save")
            .join(save_name)
            .join(format!("{}", slot))
            .with_extension("save");
        Ok(path)
    } else {
        nes_err!("failed to create save path for {:?}", path.as_ref())
    }
}
