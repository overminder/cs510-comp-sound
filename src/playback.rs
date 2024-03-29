use portaudio as pa;
use crate::types::SoundRef;
use std::mem;

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

pub fn play_def(sound: impl SoundRef) -> Result<(), pa::Error> {
    play(&Settings::default(), sound)
}

pub fn play(settings: &Settings,
            sound: impl SoundRef) -> Result<(), pa::Error> {

    // We know that the sound will not be used after this function returns,
    // so this cast of lifetime is valid.
    let bsound: Box<dyn SoundRef> = Box::new(sound);
    let mut sound: Box<dyn SoundRef> = unsafe { mem::transmute(bsound) };

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
