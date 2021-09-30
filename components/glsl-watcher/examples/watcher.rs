use std::sync::mpsc::channel;
use glsl_watcher::watch;

pub fn main() {
    println!("glsl loader");

    let (sender, _receiver) = channel();

    watch(
        sender,
        "./examples",
        "base.vert",
        "base.frag",
    );
}

