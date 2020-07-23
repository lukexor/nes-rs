//! rustynes 0.6.0
//! Luke Petherbridge <me@lukeworks.tech>
//! A NES Emulator written in Rust with SDL2 and WebAssembly support
//!
//! USAGE:
//!     rustynes [path]
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! ARGS:
//!     <path>    The NES ROM to load or a directory containing `.nes` ROM files. [default: current directory]

use std::{env, path::PathBuf};
use structopt::StructOpt;
use tetanes::{
    nes::{preferences::Preferences, Nes},
    NesResult,
};

fn main() -> NesResult<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let opt = Opt::from_args();
    let prefs = Preferences::new(opt.path)?;
    let mut nes = Nes::with_prefs(prefs)?;
    nes.run().or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    })
}

/// Command-Line Options
#[derive(StructOpt, Debug)]
#[structopt(
    name = "tetanes",
    about = "A NES Emulator written in Rust with SDL2 and WebAssembly support",
    version = "0.7.0",
    author = "Luke Petherbridge <me@lukeworks.tech>"
)]
struct Opt {
    #[structopt(
        help = "The NES ROM to load or a directory containing `.nes` ROM files. [default: current directory]"
    )]
    path: Option<PathBuf>,
    #[structopt(
        long = "speed",
        default_value = "1.0",
        help = "Increase/Decrease emulation speed. (Ranges from 0.1 to 4.0)"
    )]
    speed: f32,
}
