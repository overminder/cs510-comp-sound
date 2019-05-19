
pub fn load_wav(path: &str) -> hound::Result<Vec<i16>> {
    let reader = hound::WavReader::open(path)?;

    let ss: hound::Result<Vec<i16>> = reader.into_samples::<i16>()
        .collect();

    Ok(ss?)
}

pub fn stereo_channels_iter<'a>(v: &'a [i16]) ->
    impl Iterator<Item=(i16, i16)> + 'a {
    assert_eq!(v.len() % 2, 0);
    (0..v.len() / 2).map(move|i| {
        (v[i * 2], v[i * 2 + 1])
    })
}

