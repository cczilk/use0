use rodio::{Decoder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

pub fn play_file(path: &str, sink_lock: &Arc<Mutex<Sink>>) -> Result<(), String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader)
        .map_err(|e| format!("Failed to decode audio: {}", e))?;
    let sink = sink_lock.lock()
        .map_err(|_| "Failed to lock audio sink")?;
    sink.clear();  
    sink.append(source);
    sink.play();
    Ok(())
}