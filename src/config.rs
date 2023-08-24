use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(about = "skuggbox", long_about = "Skuggbox GLSL shader viewer")]
pub struct AppConfig {
    #[arg(short, long, name = "SHADER_FILES")]
    pub files: Option<Vec<PathBuf>>,

    #[arg(short, long)]
    pub always_on_top: bool,

    #[arg(short, long, name = "NEW_FILE")]
    pub new: Option<PathBuf>,
}
