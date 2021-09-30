use glsl_watcher::watch;
use std::sync::mpsc::channel;

pub fn main() {
    println!("glsl loader");

    let (sender, _receiver) = channel();

    watch(sender, "./examples", "base.vert", "base.frag");
}
