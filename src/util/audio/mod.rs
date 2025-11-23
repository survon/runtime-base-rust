// src/audio/mod.rs
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;

pub trait AudioPlayer {
    fn play(&mut self, path: &str, repeat: bool) -> Result<(), String>;
    fn stop(&mut self, path: &str) -> Result<(), String>;
    fn set_volume(&mut self, volume: f32);
}

struct AudioJackPlayer {
    volume: f32,
    active_sinks: Arc<Mutex<HashMap<String, Arc<Sink>>>>,
}

impl AudioJackPlayer {
    pub fn new(volume: f32) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
            active_sinks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl AudioPlayer for AudioJackPlayer {
    fn play(&mut self, path: &str, repeat: bool) -> Result<(), String> {
        let path = path.to_string();
        let volume = self.volume;
        let sinks = Arc::clone(&self.active_sinks);

        // Kill prior play of same path
        if let Some(old) = sinks.lock().unwrap().remove(&path) {
            old.stop();
        }

        let sinks_clone = Arc::clone(&sinks);
        thread::spawn(move || {
            // Create OutputStream inside the thread to avoid Send issues on macOS
            let (_stream, handle) = match OutputStream::try_default() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to create audio stream: {}", e);
                    return;
                }
            };

            let sink = match Sink::try_new(&handle) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create sink: {}", e);
                    return;
                }
            };

            sink.set_volume(volume);

            let file = match File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to open audio file {}: {}", path, e);
                    return;
                }
            };

            let source = match Decoder::new(BufReader::new(file)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to decode audio file {}: {}", path, e);
                    return;
                }
            };

            if repeat {
                sink.append(source.repeat_infinite());
            } else {
                sink.append(source);
            }

            let sink_arc = Arc::new(sink);
            sinks_clone.lock().unwrap().insert(path.clone(), sink_arc.clone());

            sink_arc.sleep_until_end();
            sinks_clone.lock().unwrap().remove(&path);
        });

        Ok(())
    }

    fn stop(&mut self, path: &str) -> Result<(), String> {
        if let Some(sink) = self.active_sinks.lock().unwrap().remove(path) {
            sink.stop();
            Ok(())
        } else {
            Err("No playback active".to_string())
        }
    }

    fn set_volume(&mut self, volume: f32) {
        let v = volume.clamp(0.0, 1.0);
        self.volume = v;
        for sink in self.active_sinks.lock().unwrap().values() {
            sink.set_volume(v);
        }
    }
}

struct GpioPwmPlayer;
impl AudioPlayer for GpioPwmPlayer {
    fn play(&mut self, _: &str, _: bool) -> Result<(), String> { Err("GPIO not ready".into()) }
    fn stop(&mut self, _: &str) -> Result<(), String> { Err("GPIO not ready".into()) }
    fn set_volume(&mut self, _: f32) {}
}

#[derive(Clone)]
pub struct SurvonAudioPlayer {
    inner: Arc<Mutex<dyn AudioPlayer + Send>>,
    path: String,
}

impl std::fmt::Debug for SurvonAudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurvonAudioPlayer")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl SurvonAudioPlayer {
    pub fn new_with_audio_jack(path: &str, volume: f32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AudioJackPlayer::new(volume))),
            path: path.to_string(),
        }
    }

    pub fn new_with_gpio_pwm(path: &str, _volume: f32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(GpioPwmPlayer)),
            path: path.to_string(),
        }
    }

    pub fn play(&mut self) -> Result<(), String> {
        self.inner.lock().unwrap().play(&self.path, false)
    }

    pub fn play_looped(&mut self) -> Result<(), String> {
        self.inner.lock().unwrap().play(&self.path, true)
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.inner.lock().unwrap().stop(&self.path)
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.inner.lock().unwrap().set_volume(volume);
    }
}
