use prost_build::Config;
use std::{fs, io::Result, path::PathBuf};

fn main() -> Result<()> {
    let mut config = Config::new();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    config.compile_protos(&["src/protos/authority_certificate.proto"], &["src/protos"])?;

    if let Ok(entries) = fs::read_dir(&out_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.ends_with(".rs") {
                    let new_path = out_dir.join("authority_certificate.rs");

                    println!(
                        "cargo:warning=Renaming {:?} to authority_certificate.rs",
                        file_name_str
                    );
                    fs::rename(entry.path(), new_path)?;
                }
            }
        }
    }

    Ok(())
}
