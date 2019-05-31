use crate::types::{R, SoundRef};

pub fn save_wav(s: impl SoundRef, name: &str, channels: u16) -> R<()> {
    let spec = hound::WavSpec {
        channels: channels,
        sample_rate: 44100,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };

    // Smaller to avoid capping.
    let amplitude = 30000.0 * 256.0;

    let mut writer = hound::WavWriter::create(name, spec)?;
    for v in s {
        writer.write_sample((v * amplitude) as i32)?;
        // writer.write_sample(v)?;
    }
    Ok(())
}
