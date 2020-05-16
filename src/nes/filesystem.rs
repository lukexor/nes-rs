use crate::{map_nes_err, nes_err, NesResult};
use std::{
    io::BufWriter,
    path::{Path, PathBuf},
};

pub const CONFIG_DIR: &str = ".rustynes";

/// Creates a '.png' file
///
/// # Arguments
///
/// * `png_path` - An object that implements AsRef<Path> for the location to save the `.png`
/// file
/// * `pixels` - An array of pixel data to save in `.png` format
///
/// # Errors
///
/// It's possible for this method to fail, but instead of erroring the program,
/// it'll simply log the error out to STDERR
pub fn create_png<P: AsRef<Path>>(
    png_path: &P,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> NesResult<String> {
    let png_path = png_path.as_ref();
    let png_file = std::fs::File::create(&png_path);
    if png_file.is_err() {
        return nes_err!(
            "failed to create png file {:?}: {}",
            png_path.display(),
            png_file.err().unwrap(),
        );
    }
    let png_file = BufWriter::new(png_file.unwrap()); // Safe to unwrap
    let mut png = png::Encoder::new(png_file, width, height);
    png.set_color(png::ColorType::RGB);
    let writer = png.write_header();
    if let Err(e) = writer {
        return nes_err!("failed to save screenshot {:?}: {}", png_path.display(), e);
    }
    let result = writer.unwrap().write_image_data(&pixels);
    if let Err(e) = result {
        return nes_err!("failed to save screenshot {:?}: {}", png_path.display(), e);
    }
    Ok(format!("{}", png_path.display()))
}

/// Searches for valid NES rom files ending in `.nes`
///
/// If rom_path is a `.nes` file, uses that
/// If no arg[1], searches current directory for `.nes` files
pub fn find_roms<P: AsRef<Path>>(path: &P) -> NesResult<Vec<PathBuf>> {
    use std::ffi::OsStr;
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
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("./"));
    path.push(CONFIG_DIR);
    path.push("sram");
    path.push(save_name);
    path.set_extension("sram");
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
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("./"));
        path.push(CONFIG_DIR);
        path.push("save");
        path.push(save_name);
        path.push(format!("{}", slot));
        path.set_extension("save");
        Ok(path)
    } else {
        nes_err!("failed to create save path for {:?}", path.as_ref())
    }
}
