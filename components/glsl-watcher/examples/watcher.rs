use std::path::PathBuf;
use std::sync::mpsc::channel;

use glsl_watcher::watch_all;

pub fn main() {
    println!("glsl loader");

    let (sender, _receiver) = channel();
    watch_all(sender, vec![PathBuf::from("./examples/base.frag")]);
}
