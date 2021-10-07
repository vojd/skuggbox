use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "skuggbox", about = "Skuggbox usage")]
pub struct Config {
    #[structopt(long, short)]
    pub fragment_shader: PathBuf,
}
