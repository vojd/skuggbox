use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use glsl_watcher::watch_all;

pub fn main() {
    println!("glsl loader");

    let (sender, receiver) = channel();
    watch_all(sender, vec![PathBuf::from("./examples/base.frag")]);

    loop {
        for res in receiver.recv() {
            println!(">> {:?}", res);
        }

        if let Ok(s) = receiver.try_recv() {
            println!("change detected {:?}", s);
        }

        thread::sleep(Duration::from_millis(10));
    }

    // sender_thread.join().expect("Sender thread panic");
}
