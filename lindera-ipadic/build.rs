use std::env;
use std::error::Error;
use std::fs::{rename, File};
use std::path::Path;

use flate2::read::GzDecoder;
use reqwest;
use tar::Archive;
use tokio;
use tokio::fs::File as TokioFile;
use tokio::prelude::*;

use lindera_ipadic_builder::build;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap(); // ex) target/debug/build/<pkg>/out

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let ipadic_ver = "2.7.0-20070801";
    let file_name = format!("mecab-ipadic-{}.tar.gz", ipadic_ver);

    let dest_path = Path::new(&out_dir).join(&file_name);
    if !dest_path.exists() {
        let tmp_path = Path::new(&out_dir).join(file_name + ".download");

        // Download a tarball
        let download_url =
            "https://drive.google.com/uc?export=download&id=0B4y35FiV1wh7MWVlSDBCSXZMTXM";
        let mut resp = reqwest::get(download_url).await.unwrap();

        // Save a ttarball
        let mut dest = TokioFile::create(&tmp_path).await.unwrap();
        while let Some(chunk) = resp.chunk().await.unwrap() {
            dest.write_all(&chunk).await?;
        }
        rename(tmp_path, &dest_path).expect("Failed to rename temporary file");
    }

    // Decompress a tarball
    let tar_gz = File::open(&dest_path).unwrap();
    let gzdecoder = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(gzdecoder);
    archive.unpack(&out_dir).unwrap();

    // Build dictionary
    let input_dir = Path::new(&out_dir).join(format!("mecab-ipadic-{}", ipadic_ver));
    let output_dir = Path::new(&out_dir).join("lindera-ipadic");
    build(input_dir.to_str().unwrap(), output_dir.to_str().unwrap()).unwrap();

    Ok(())
}
