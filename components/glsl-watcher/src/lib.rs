#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

use notify::{Op, raw_watcher, RawEvent, RecursiveMode, Watcher};

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
pub fn watch_all(sender: Sender<PathBuf>, files: &Vec<String>) {
    let (watch_sender, watch_receiver) = channel();
    let mut watcher = raw_watcher(watch_sender).unwrap();

    let paths : HashSet<PathBuf> = HashSet::from_iter(files.iter().map(|file| PathBuf::from(file)).into_iter());
    let mut dir_set : HashSet<PathBuf> = HashSet::new();

    for path in paths {
        let parent_path = match path.as_path().parent() {
            None => PathBuf::from("./"),
            Some(parent) => PathBuf::from(parent)
        };

        let mut insert = true;

        dir_set.retain(|existing_dir| {
            if existing_dir.starts_with(&parent_path) {
                // a directory in the set is a child to the current directory, remote it
                return false;
            }

            if parent_path.starts_with(&existing_dir) {
                // the directory is already represented in the set
                // either by the exact path or by a parent directory, don't add the new path
                insert = false;
            }

            return true;
        });

        if insert {
            dir_set.insert(parent_path);
        }
    }

    println!("Watching files shaders in:");
    for dir in &dir_set {
        watcher.watch(dir.as_path(), RecursiveMode::Recursive).unwrap();
        println!("   {:?}", dir);
    }

    loop {
        // NOTE: It's likely that a change to a file will trigger two successive WRITE events
        let changed_file = match watch_receiver.recv() {
            Ok(RawEvent {
                   path: Some(path),
                   op: Ok(op),
                   ..
               }) => {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if op == Op::WRITE && dir_set.contains(&path) {
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
            sender.send(changed_file.unwrap()).unwrap();
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

