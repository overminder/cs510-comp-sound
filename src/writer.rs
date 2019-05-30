use crate::types::{R, SoundRef};

pub fn save_wav(s: impl SoundRef, name: &str) -> R<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Smaller to avoid capping.
    let amplitude = 30000.0;

    let mut writer = hound::WavWriter::create(name, spec)?;
    for v in s {
        writer.write_sample((v * amplitude) as i16)?;
    }
    Ok(())
}
