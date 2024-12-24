use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Read version from Cargo.toml environment variable
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    // Write the version to a file in the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version.rs");
    let mut f = File::create(&dest_path).unwrap();

    write!(f, "const VERSION: &str = \"{}\";", version).unwrap();
}
