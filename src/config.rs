use std::path::PathBuf;

#[derive(clap::Parser, Debug, Clone)]
#[clap(name = "skuggbox", about = "Skuggbox GLSL shader viewer")]
pub struct Config {
    #[clap(short, long, name = "SHADER_FILES", parse(from_os_str))]
    pub files: Option<Vec<PathBuf>>,

    #[clap(short, long)]
    pub always_on_top: bool,

    #[clap(short, long, name = "NEW_FILE", parse(from_os_str))]
    pub new: Option<PathBuf>,
}
