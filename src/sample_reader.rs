use crate::types::R;
use rimd::SMF;

pub fn load_wav(path: &str) -> R<Vec<i16>> {
    let reader = hound::WavReader::open(path)?;

    let ss: hound::Result<Vec<i16>> = reader.into_samples::<i16>()
        .collect();

    Ok(ss?)
}

pub fn load_flac(path: &str) -> R<(Vec<f32>, Vec<f32>)> {
    let mut r = claxon::FlacReader::open(path)?;
    let max_block_len = r.streaminfo().max_block_size as usize * 2;
    let num_samples = r.streaminfo().samples.unwrap() as usize;
    let mut ch0 = Vec::with_capacity(num_samples);
    let mut ch1 = Vec::with_capacity(num_samples);
    let mut buf = Vec::with_capacity(max_block_len);
    let mut blocks = r.blocks();

    loop {
        match blocks.read_next_or_eof(buf) {
            Ok(Some(block)) => {
                ch0.extend(block.channel(0)
                           .iter()
                           .cloned()
                           .map(pcm24_to_f32_norm));
                ch1.extend(block.channel(1)
                           .iter()
                           .cloned()
                           .map(pcm24_to_f32_norm));
                buf = block.into_buffer();
            },
            Ok(None) => break, // End of file.
            Err(_) => panic!("failed to decode")
        }
    }

    Ok((ch0, ch1))
}

fn pcm24_to_f32_norm(x: i32) -> f32 {
    let mult = (i32::max_value() >> 8) as f32;
    x as f32 / mult
}

pub fn read_midi(path: &str) -> R<SMF> {
    let f = SMF::from_file(path.as_ref())?;
    Ok(f)
}

pub fn stereo_channels_iter<'a>(v: &'a [i16]) ->
    impl Iterator<Item=(i16, i16)> + 'a {
    assert_eq!(v.len() % 2, 0);
    (0..v.len() / 2).map(move|i| {
        (v[i * 2], v[i * 2 + 1])
    })
}

