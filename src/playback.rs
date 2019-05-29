use portaudio as pa;
use crate::types::Sound;

pub struct Settings {
    pub channels: i32,
    pub sample_rate: f64,
    pub frames_per_buffer: u32,
}

impl Settings {
    pub fn default() -> Self {
        Self {
            channels: 1,
            sample_rate: 44_100.0,
            frames_per_buffer: 64,
        }
    }
}

pub fn play_def(mut sound: impl Sound) -> Result<(), pa::Error> {
    play(&Settings::default(), sound)
}

pub fn play(settings: &Settings,
            mut sound: impl Sound) -> Result<(), pa::Error> {
    let pa = pa::PortAudio::new()?;

    let mut pa_settings = pa.default_output_stream_settings(
        settings.channels,
        settings.sample_rate,
        settings.frames_per_buffer)?;
    pa_settings.flags = pa::stream_flags::CLIP_OFF;

    let callback = move |args: pa::OutputStreamCallbackArgs<_>| {
        let buffer = args.buffer;

        for b in buffer {
            if let Some(v) = sound.next() {
                *b = v;
            } else {
                return pa::Complete
            }
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(pa_settings, callback)?;

    stream.start()?;

    while stream.is_active()? {
        pa.sleep(100);
    }

    stream.stop()?;
    stream.close()?;

    println!("Done playback");

    Ok(())
}
