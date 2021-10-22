use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[clap(name = "skuggbox", about = "Skuggbox GLSL shader viewer")]
pub struct Config {
    /// GLSL shader file to load
    #[clap(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,

}