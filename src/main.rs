extern crate gl;
extern crate glutin;
extern crate winit;

use std::fs;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use simple_logger::SimpleLogger;

use skuggbox::{app::App, config::Config};

/// Creates a new default shader at the gives path
fn create_new_default_shader(path: PathBuf) -> std::io::Result<u64> {
    if path.exists() {
        log::warn!("File already exists. Not overwriting it to save your life :D");
        exit(1);
    }
    fs::copy("shaders/init.glsl", path)
}

fn main() -> anyhow::Result<(), anyhow::Error> {
    SimpleLogger::new().init().unwrap();

    // Parse command line arguments using `structopt`
    let config = Config::parse();
    let mut app = App::from_config(config.clone());

    if let Some(new_file) = config.clone().new {
        log::info!("creating new shader at {:?}", new_file);
        match create_new_default_shader(new_file) {
            Ok(_) => log::info!("Done"),
            Err(err) => {
                log::error!("{:?}", err);
                exit(1);
            }
        }
    }

    if config.files.is_some() {
        log::info!("loading existing shader");
        app.run(config);
    }
    Ok(())
}
