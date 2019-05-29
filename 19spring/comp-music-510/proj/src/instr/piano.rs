use crate::types::{R, Sound};
use crate::sample_reader::{
    load_wav,
    stereo_channels_iter,
};
use crate::soundprim::Envelope;
use std::collections::HashMap;
use std::path::Path;

type NoteMap = HashMap<i32, Vec<f32>>;

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
            notes.insert(key, ss);
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
            notes.insert(key, ss);
        }
    }
    Ok(notes)
}

impl Piano {
    pub fn load(base_path: &str) -> R<Self> {
        let notes = load_mono_mf(base_path)?;
        // let notes = load_raw_mf_to_mono(base_path)?;
        Ok(Piano {
            notes
        })
    }

    pub fn syn(&self, key: i32, amp: f64) -> impl Sound {
        // XXX not quite performant
        let ss = self.notes[&key].to_owned();
        let dur = ss.len() as f64 / 44100.0;

        let mut env = Envelope::fast_release();
        env.amp = amp;
        env.mult(ss.into_iter(), dur)
    }
}

