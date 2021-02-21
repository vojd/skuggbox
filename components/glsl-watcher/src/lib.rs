#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};

pub fn watch(sender: Sender<bool>, dir: &str, vs: &str, fs: &str) {
    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx).unwrap();
    watcher.watch("./", RecursiveMode::Recursive).unwrap();
    println!("Watching shaders in {}", dir);

    loop {
        // NOTE: It's likely that a change to a file will trigger two successive WRITE events
        let changed_file = match rx.recv() {
            Ok(RawEvent {
                path: Some(path),
                op: Ok(op),
                ..
            }) => {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if op == Op::WRITE && (file_name == vs || file_name == fs) {
                    println!("change in: {:?}", file_name);
                    Some(path)
                } else {
                    None
                }
            }

            Ok(event) => {
                println!("broken event: {:?}", event);
                None
            }
            Err(e) => {
                println!("watch error: {:?}", e);
                None
            }
        };

        if changed_file.is_some() {
            sender.send(true).unwrap();
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}
