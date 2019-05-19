use music_syn::{
    sample_reader::*,
    playback::*,
    soundprim::*,
    types::*,
};

fn i16_to_f32_norm(x: i16) -> f32 {
    (x as f32 / i16::max_value() as f32) * 6.
}

fn merge_stereo(v: &[i16]) -> Vec<f32> {
    stereo_channels_iter(&v)
        .map(|(l, r)| (i16_to_f32_norm(l) + i16_to_f32_norm(r)) / 2.0)
        .collect()
}

fn main() -> R<()> {
    let w2 = load_wav("samples/Piano.mf.C4.wav")?;
    let w = merge_stereo(&w2);

    // Kind of need to get this offset...
    let offset = 15000;
    let N = 5000;
    let env = Envelope::default();
    let piece = w[offset..offset+N].to_owned();
    let loped: Vec<f32> = env
        .mult(piece.to_owned().into_iter(), N as f64 / SAMPLE_RATE)
        .collect();
    let out: Vec<f32> = 
        loped.iter()
            .chain(loped.iter())
            .chain(loped.iter())
            .cloned()
            .collect();

    let mut s = Settings::default();
    s.channels = 1;

    play(&s, out.into_iter())?;

    // stereo_channels_iter(&w).map(
    //     |(l, r)|
    //     l / 

    Ok(())
}
