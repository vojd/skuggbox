#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

/// Using `notify` to watch for changes to files in any directories
/// where the files resides.
/// `sender` - std::sync::mpsc::channel
/// `files` - Which files to watch
///
pub fn watch_all(sender: Sender<PathBuf>, files: Vec<PathBuf>) {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    let directories: Vec<PathBuf> = files
        .iter()
        .filter_map(|p| fs::canonicalize(p).ok())
        .collect();

    println!("Watching files shaders in:");
    for dir in &directories {
        watcher
            .watch(dir.as_path(), RecursiveMode::Recursive)
            .unwrap(); // TODO: Replace with GLSLWatcherError
        println!("   {:?}", dir);
    }

    watch_loop(sender, rx, directories);
}

fn watch_loop(
    sender: Sender<PathBuf>,
    watch_receiver: Receiver<notify::Result<notify::Event>>,
    _directories: Vec<PathBuf>,
) {
    // TODO: Handle the different types of event for example removing file(s)
    // EventKind::Any => {}
    // EventKind::Access(_) => {}
    // EventKind::Create(_) => {}
    // EventKind::Modify(_) => {}
    // EventKind::Remove(_) => {}
    // EventKind::Other => {}
    loop {
        while let Ok(res) = watch_receiver.recv() {
            if let Ok(event) = res {
                let path = event.paths.first().unwrap();
                sender
                    .send(path.to_owned().canonicalize().unwrap())
                    .unwrap();
            }
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}
