use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify::{raw_watcher, Op, RawEvent, RecommendedWatcher, RecursiveMode, Watcher};

pub struct WatcherService {
    started: AtomicBool,
    on_change_sender: Sender<PathBuf>,
    watcher: RecommendedWatcher,
    files_mutex: Arc<Mutex<Vec<PathBuf>>>,
    watch_receiver: Arc<Mutex<Receiver<RawEvent>>>,
}

impl WatcherService {
    pub fn new(on_change_sender: Sender<PathBuf>) -> Self {
        let (watch_sender, watch_receiver) = channel();
        let watcher = raw_watcher(watch_sender).unwrap();

        Self {
            started: AtomicBool::new(false),
            on_change_sender,
            watcher,
            files_mutex: Arc::new(Mutex::new(Vec::new())),
            watch_receiver: Arc::new(Mutex::new(watch_receiver)),
        }
    }

    pub fn is_watching(&self, file: &Path) -> bool {
        let files_watched = self.files_mutex.lock().unwrap();
        files_watched.contains(&file.to_path_buf())
    }

    pub fn watch(&mut self, files: Vec<PathBuf>) {
        if self.started.load(Ordering::Relaxed) {
            self.watch_all(files);
            return;
        }

        // no service is currently running so just add the files to the list
        let mut files_watched = self.files_mutex.lock().unwrap();
        for file in files.iter() {
            if !files_watched.contains(file) {
                files_watched.push(file.to_path_buf());
            }
        }
    }

    pub fn unwatch(&mut self, files: Vec<PathBuf>) {
        if self.started.load(Ordering::Relaxed) {
            for file in files.iter() {
                match self.watcher.unwatch(file) {
                    Ok(_) => {
                        println!("Unwatching file {:?}", file);
                    }
                    Err(_) => {
                        println!("Error: Can't unwatch file {:?}", file);
                    }
                }
            }
            return;
        }

        // no service is currently running so just remove the files from the list
        let mut files_watched = self.files_mutex.lock().unwrap();
        files_watched.retain(|path| files.contains(path));
    }

    fn watch_all(&mut self, files: Vec<PathBuf>) {
        for file in files.iter() {
            match self.watcher.watch(file, RecursiveMode::NonRecursive) {
                Ok(_) => {
                    println!("Watching file {:?}", file);
                }
                Err(_) => {
                    println!("Error: Can't watch file {:?}", file);
                }
            }
        }
    }

    pub fn start(&mut self) {
        if self.started.load(Ordering::Relaxed) {
            // the service had already been started, just return because we're nice people
            return;
        }
        self.started.store(true, Ordering::Relaxed);

        let files_watched = self.files_mutex.lock().unwrap().iter().cloned().collect();

        self.watch_all(files_watched);
        self.threaded_watch_loop();
    }

    fn threaded_watch_loop(&mut self) {
        let receiver = Arc::clone(&self.watch_receiver);
        let change_sender = self.on_change_sender.clone();

        let _ = thread::spawn(move || {
            let locked_receiver = receiver.lock().unwrap();

            loop {
                // NOTE: It's likely that a change to a file will trigger two successive WRITE events
                let changed_file = match locked_receiver.recv() {
                    Ok(RawEvent {
                        path: Some(path),
                        op: Ok(op),
                        ..
                    }) => {
                        if op == Op::WRITE {
                            println!("change in: {:?}", path);
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
                    change_sender.send(cf).unwrap();
                }

                thread::sleep(Duration::from_millis(50));
            }
        });
    }
}
