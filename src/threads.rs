use std::sync::{Arc, Mutex};
use std::sync::atomic::{Ordering, AtomicBool};
use std::{thread, time::{Duration, Instant}};
use still_alive::{data, Event};

use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source};

use crossterm::ExecutableCommand;

// Cursor blinking
pub fn cursor(cursor_visible: Arc<AtomicBool>) {
    thread::spawn(move || loop {
        cursor_visible.fetch_xor(true, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(250));
    });
}

// Right panel
pub fn secondary(text: Arc<Mutex<String>>, start_time: Instant) {
    thread::spawn(move || {
        for line in data::right::TEXT.iter() {
            let elapsed = start_time.elapsed();
            if elapsed < line.start {
                thread::sleep(line.start - elapsed);
            }

            line.write(&text);
        }
    });
}

// Left panel & event handling
pub fn main(left_text: Arc<Mutex<String>>, right_text: Arc<Mutex<String>>, ascii: Arc<Mutex<String>>) {
    thread::spawn(move || {
        let start_time = Instant::now();
        secondary(Arc::clone(&right_text), start_time);

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        for line in data::left::TEXT.iter() {
            let elapsed = start_time.elapsed();
            if elapsed < line.start {
                thread::sleep(line.start - elapsed);
            }

            match &line.event {
                Some(Event::StartMusic) => {
                    let file = BufReader::new(File::open("still-alive.wav").unwrap());
                    let source = Decoder::new(file).unwrap();
                    stream_handle.play_raw(source.convert_samples()).unwrap();
                },
                Some(Event::ClearScreen) => left_text.lock().unwrap().clear(),
                Some(Event::Draw(art)) => *ascii.lock().unwrap() = String::from(*art),
                Some(Event::Exit) => {
                    crate::disable_raw_mode().unwrap();
                    crate::stdout().execute(crate::LeaveAlternateScreen).unwrap();
                    std::process::exit(0)
                },
                None => ()
            }

            line.write(&left_text);
        }
    });
}
