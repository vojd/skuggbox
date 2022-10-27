use std::borrow::Borrow;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use notify::{raw_watcher, Op, RawEvent, RecommendedWatcher, RecursiveMode, Watcher};

pub struct WatcherService {
    on_change_sender: Arc<Mutex<Sender<PathBuf>>>,
    files_mutex: Arc<Mutex<Vec<PathBuf>>>,
    watcher: RecommendedWatcher,
    watch_receiver: Mutex<Receiver<RawEvent>>,
    started: AtomicBool,
}

impl WatcherService {
    pub fn new(on_change_sender: Sender<PathBuf>) -> Self {
        let (watch_sender, watch_receiver) = channel();
        let watcher = raw_watcher(watch_sender).unwrap();

        Self {
            on_change_sender: Arc::new(Mutex::new(on_change_sender)),
            files_mutex: Arc::new(Mutex::new(Vec::new())),
            watcher,
            watch_receiver: Mutex::new(watch_receiver),
            started: AtomicBool::new(false),
        }
    }

    pub fn is_watching(&self, file: &PathBuf) -> bool {
        let files_watched = self.files_mutex.lock().unwrap();
        files_watched.contains(file)
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
                files_watched.push(file.clone());
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

    pub fn start(&'static mut self) {
        if self.started.load(Ordering::Relaxed) {
            // the service had already been started, just return because we're nice people
            return;
        }

        let files_watched = self
            .files_mutex
            .lock()
            .unwrap()
            .iter()
            .map(|path| path.clone())
            .collect();

        self.watch_all(files_watched);
        self.threaded_watch_loop();
    }

    fn threaded_watch_loop(&'static self) {
        let receiver = self.watch_receiver.borrow();
        let change_sender: &Mutex<Sender<PathBuf>> = self.on_change_sender.borrow();

        let _ = thread::spawn(move || {
            let locked_receiver = receiver.lock().unwrap();
            let locked_sender = change_sender.lock().unwrap();

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
                    locked_sender.send(cf).unwrap();
                }

                std::thread::sleep(Duration::from_millis(50));
            }
        });
    }
}
