use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "skuggbox",
    about = "Skuggbox usage",
)]
pub struct Config {
    /// Shader to load
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,
}
