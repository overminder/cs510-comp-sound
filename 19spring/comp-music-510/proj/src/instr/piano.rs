use crate::types::{R, Sound};
use crate::sample_reader::load_wav;
use crate::soundprim::Envelope;
use std::collections::HashMap;
use std::path::Path;

pub struct Piano {
    notes: HashMap<i32, Vec<f32>>,
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

impl Piano {
    pub fn load(base_path: &str) -> R<Self> {
        let mut me = Piano {
            notes: HashMap::new(),
        };

        for key in -36..48 {
            let path = format!("{}/{}.wav", base_path, key_to_name(key));
            if Path::new(&path).exists() {
                let ss = load_wav(&path)?;
                let ss = ss.into_iter().map(i16_to_f32_norm).collect();
                me.notes.insert(key, ss);
            }
        }
        Ok(me)
    }

    pub fn syn(&self, key: i32, amp: f64) -> impl Sound {
        let ss = self.notes[&key].to_owned();
        let dur = ss.len() as f64 / 44100.0;

        let mut env = Envelope::fast_release();
        env.amp = amp;
        env.mult(ss.into_iter(), dur)
    }
}

