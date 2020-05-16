use tetanes::{nes::Nes, NesResult};

fn main() -> NesResult<()> {
    let nes = Nes::new();
    nes.run().or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    })
}
