# Some tests rely on deterministic RAM state
cargo build --release
find tests/ -iname '*.nes' -exec target/release/tetanes --speed 4 {} \;
