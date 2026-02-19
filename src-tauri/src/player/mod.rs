pub mod engine;

use std::sync::{Arc, Mutex};
use rodio::{OutputStreamBuilder, Sink};

pub struct PlayerState {
    pub sink: Arc<Mutex<Sink>>,
    pub _stream: rodio::OutputStream,
}

impl PlayerState {
    pub fn new() -> Self {
        let stream = OutputStreamBuilder::open_default_stream()
            .expect("Could not find an audio output device");
        let sink = Sink::connect_new(&stream.mixer());
        Self {
            sink: Arc::new(Mutex::new(sink)),
            _stream: stream,
        }
    }
}