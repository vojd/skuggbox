use minimp3::{Decoder, Error, Frame};
use sdl2::audio::{AudioCallback, AudioDevice};
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct FrameWriter {
    frames: Vec<i16>,
    current_sample: usize,
    receiver: Receiver<f32>,
}

pub struct AudioPlayer {
    device: Option<AudioDevice<FrameWriter>>,
    sender: Option<Sender<f32>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            device: None,
            sender: None,
        }
    }

    pub fn load(&mut self, mp3_path: PathBuf) {
        log::info!("Loading mp3 file: {:?}", mp3_path);
        let mut decoder = Decoder::new(File::open(mp3_path).unwrap());
        let mut frames = Vec::new();

        loop {
            match decoder.next_frame() {
                Ok(Frame { data, channels, .. }) => {
                    for i in 0..(data.len() / channels) {
                        frames.push(data[i * channels]);
                        frames.push(data[i * channels + 1]);
                    }
                }
                Err(Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
        }

        // instantiate the audio device
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None, // default sample size
        };

        let (sender, receiver) = channel();

        let frame_writer = FrameWriter {
            frames,
            current_sample: 0,
            receiver,
        };
        let device = audio_subsystem
            .open_playback(None, &desired_spec, |_spec| {
                log::info!("Audio spec: {:?}", _spec);
                frame_writer
            })
            .unwrap();

        self.device = Some(device);
        self.sender = Some(sender);
    }

    pub fn play(&mut self) {
        if self.device.is_none() {
            return;
        }
        log::info!("Playing audio");
        self.device.as_ref().unwrap().resume();
    }

    pub fn pause(&mut self) {
        if self.device.is_none() {
            return;
        }
        log::info!("Pausing audio");
        self.device.as_ref().unwrap().pause();
    }

    pub fn seek(&mut self, time: f32) {
        if self.sender.is_none() {
            return;
        }
        self.sender.as_ref().unwrap().send(time).unwrap();
    }
}

impl AudioCallback for FrameWriter {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        loop {
            match self.receiver.try_recv() {
                Ok(time) => {
                    self.current_sample = (time * 2.0 *  44100.0) as usize;
                }
                Err(_) => break,
            }
        }

        let max_length = self.frames.len();

        if self.current_sample >= max_length {
            for x in out.iter_mut() {
                *x = 0;
            }
            return;
        }

        // Generate a square wave
        for x in out.iter_mut() {
            if self.current_sample >= max_length {
                self.current_sample = 0;
            } else {
                *x = self.frames[self.current_sample];
                self.current_sample += 1;
            }
        }
    }
}
