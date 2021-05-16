pub mod cache;

use std::{fs::File, io::Read, path::Path};

use fontdue::{Font, FontSettings};
type Result<T> = std::result::Result<T, String>;

pub fn font_from_file<P: AsRef<Path>>(path: P, settings: FontSettings) -> Result<Font> {
    let mut file = File::open(path).map_err(|err| err.to_string())?;
    let mut bytes = Vec::with_capacity(512);
    file.read_to_end(&mut bytes).map_err(|err| err.to_string())?;
    Font::from_bytes(bytes, settings).map_err(str::to_string)
}
