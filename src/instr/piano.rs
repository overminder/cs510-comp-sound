use crate::types::{R, Sound};
use crate::sample_reader::{
    load_wav,
    stereo_channels_iter,
};
use crate::soundprim::Envelope;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

type NoteMap = HashMap<i32, Rc<Vec<f32>>>;

pub struct Piano {
    notes: NoteMap,
}


const NOTE_NAMES: &'static [&'static str] = &[
    "C", "Db", "D", "Eb", "E", "F",
    "Gb", "G", "Ab", "A", "Bb", "B",
];

// Convert semitone wrt C4 (i.e. 0 = C4) to pitch name.
fn key_to_name(k: i32) -> String {
    // Such that 0 is C1 rather than C4.
    let k = k + 36;
    let name = NOTE_NAMES[(k % 12) as usize];
    let nth_octave = 1 + k / 12;
    format!("{}{}", name, nth_octave)
}

fn i16_to_f32_norm(x: i16) -> f32 {
    (x as f32 / i16::max_value() as f32) * 6.
}

fn pcm24_to_f32_norm(x: i32) -> f32 {
    let mult = (i32::max_value() >> 8) as f32;
    x as f32 / mult
}

fn merge_stereo(v: &[i16]) -> Vec<f32> {
    stereo_channels_iter(&v)
        .map(|(l, r)| (i16_to_f32_norm(l) + i16_to_f32_norm(r)) / 2.0)
        .collect()
}

fn load_mono_mf(base_path: &str) -> R<NoteMap> {
    let mut notes = NoteMap::new();

    for key in -36..48 {
        let path = format!("{}/{}.wav", base_path, key_to_name(key));
        if Path::new(&path).exists() {
            let ss = load_wav(&path)?;
            let ss = ss.into_iter().map(i16_to_f32_norm).collect();
            notes.insert(key, Rc::new(ss));
        }
    }
    Ok(notes)
}

fn load_raw_mf_to_mono(base_path: &str) -> R<NoteMap> {
    let mut notes = NoteMap::new();

    for key in -36..48 {
        let path = format!("{}/../Piano.mf.{}.wav", base_path, key_to_name(key));
        if Path::new(&path).exists() {
            let ss = load_wav(&path)?;
            let ss = merge_stereo(&ss[..200000]);
            // let ss = ss.into_iter().map(i16_to_f32_norm).collect();
            notes.insert(key, Rc::new(ss));
        }
    }
    Ok(notes)
}

fn load_normed_flac(base_path: &str, dynamics: &str) -> R<(NoteMap, NoteMap)> {
    let mut notes0 = NoteMap::new();
    let mut notes1 = NoteMap::new();

    for key in -36..48 {
        let path = format!("{}/{}.{}.flac",
                           base_path, key_to_name(key), dynamics);
        if Path::new(&path).exists() {
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
            // println!("Load {} -> {} samples", &key_to_name(key), num_samples);
            notes0.insert(key, Rc::new(ch0));
            notes1.insert(key, Rc::new(ch1));
        }
    }
    Ok((notes0, notes1))
}

impl Piano {
    pub fn load(base_path: &str) -> R<Self> {
        // let notes = load_mono_mf(base_path)?;
        let (notes0, notes1) = load_normed_flac(base_path, "mf")?;
        // let notes = load_raw_mf_to_mono(base_path)?;
        Ok(Piano {
            notes: notes0
        })
    }

    pub fn syn(&self, key: i32, amp: f64) -> impl Sound {
        let ss = self.notes[&key].clone();
        let len = ss.len();
        let dur = len as f64 / 44100.0;

        let mut env = Envelope::fast_release();
        // TODO: Mix ff, mf, pp by amp.
        env.amp = amp;
        env.mult((0..len).map(move |ix| ss[ix]), dur)
    }
}

