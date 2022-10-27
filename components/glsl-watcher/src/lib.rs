#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

pub mod watcher_service;
pub use watcher_service::*;

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};

/// Using `notify` to watch for changes to any defined shader file
/// `sender` - std::sync::mpsc::channel
/// `dir` - Which dir to find the files in
/// `vs` - vertex shader name located in `dir`
/// `fs` - fragment shader name located in `dir
///
pub fn watch(sender: Sender<bool>, dir: &str, vs: &str, fs: &str) {
    let (watch_sender, watch_receiver) = channel();
    let mut watcher = raw_watcher(watch_sender).unwrap();
    watcher.watch("./", RecursiveMode::Recursive).unwrap();
    println!("Watching shaders in {}", dir);

    loop {
        // NOTE: It's likely that a change to a file will trigger two successive WRITE events
        let changed_file = match watch_receiver.recv() {
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

/// Using `notify` to watch for changes to any defined shader file
/// `sender` - std::sync::mpsc::channel
/// `files` - Which files to watch
///
pub fn watch_all(sender: Sender<PathBuf>, files: Vec<PathBuf>) {
    let (watch_sender, watch_receiver) = channel();
    let mut watcher = raw_watcher(watch_sender).unwrap();

    let directories: Vec<PathBuf> = files
        .iter()
        .filter_map(|p| fs::canonicalize(p).ok())
        .collect();

    println!("Watching files shaders in:");
    for dir in &directories {
        watcher
            .watch(dir.as_path(), RecursiveMode::Recursive)
            .unwrap();
        println!("   {:?}", dir);
    }

    watch_loop(sender, watch_receiver, directories);
}

fn watch_loop(
    sender: Sender<PathBuf>,
    watch_receiver: Receiver<RawEvent>,
    directories: Vec<PathBuf>,
) {
    loop {
        // NOTE: It's likely that a change to a file will trigger two successive WRITE events
        let changed_file = match watch_receiver.recv() {
            Ok(RawEvent {
                path: Some(path),
                op: Ok(op),
                ..
            }) => {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                println!("on change in: {:?}", path.to_str().unwrap());
                if op == Op::WRITE && directories.contains(&path) {
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

        if let Some(cf) = changed_file {
            sender.send(cf).unwrap();
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}
