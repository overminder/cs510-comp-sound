use crate::types::{R, Sound};
use crate::soundprim::Envelope;
use crate::sample_reader::load_flac;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

type NoteMap = HashMap<i32, Rc<Vec<f32>>>;

pub struct Piano {
    // 0/1/2: pp, mf, ff
    notes: Vec<NoteMap>,
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

fn load_normed_flac(base_path: &str, dynamics: &str) -> R<(NoteMap, NoteMap)> {
    let mut notes0 = NoteMap::new();
    let mut notes1 = NoteMap::new();

    for key in -36..48 {
        let path = format!("{}/{}.{}.flac",
                           base_path, key_to_name(key), dynamics);
        if Path::new(&path).exists() {
            let (ch0, ch1) = load_flac(&path)?;
            notes0.insert(key, Rc::new(ch0));
            notes1.insert(key, Rc::new(ch1));
        }
    }
    Ok((notes0, notes1))
}

impl Piano {
    pub fn load(base_path: &str) -> R<(Self, Self)> {
        let (pp0, pp1) = load_normed_flac(base_path, "pp")?;
        let (mf0, mf1) = load_normed_flac(base_path, "mf")?;
        let (ff0, ff1) = load_normed_flac(base_path, "ff")?;

        Ok((Piano { notes: vec![pp0, mf0, ff0] },
            Piano { notes: vec![pp1, mf1, ff1] }))
    }

    pub fn syn(&self, key: i32, amp: f64) -> impl Sound {
        self.syn_by_nomix(key, amp)
    }

    #[allow(unused)]
    fn syn_by_nomix(&self, key: i32, amp: f64) -> impl Sound {
        let pp_max = 32.0 / 128.0;
        let mf_max = 80.0 / 128.0;
        let ff_max = 112.0 / 128.0;
        let dyn_ix = if amp < (mf_max + pp_max) / 2.0 {
            0
        } else if amp > (mf_max + ff_max) / 2.0 {
            2
        } else {
            1
        };

        let ss = self.notes[dyn_ix][&key].clone();

        let len = ss.len();
        let dur = len as f64 / 44100.0;

        let mut env = Envelope::fast_release();
        env.amp = amp;
        env.mult((0..len).map(move |ix| ss[ix]), dur)
    }

    #[allow(unused)]
    fn syn_by_mix(&self, key: i32, amp: f64) -> impl Sound {
        // amp 32 80 112
        // ff  1   0
        // mf  0   1   0
        // pp      0   1
        // Between: interpolate both samples 
        let pp_max = 32.0;
        let mf_max = 80.0;
        let ff_max = 112.0;
        let amp_max = 128.0;

        let pp = self.notes[0][&key].clone();
        let mf = self.notes[1][&key].clone();
        let ff = self.notes[2][&key].clone();

        let ampf = (amp as f32) * amp_max;
        let pp_amp = if ampf < pp_max {
            1.0
        } else if ampf > mf_max {
            0.0
        } else {
            1.0 - (ampf - pp_max) / (mf_max - pp_max)
        };
        
        let mf_amp = if ampf < pp_max {
            0.0
        } else if ampf > ff_max {
            0.0
        } else if ampf < mf_max {
            (ampf - pp_max) / (mf_max - pp_max)
        } else {
            1.0 - (ampf - mf_max) / pp_max
        };

        let ff_amp = if ampf < ff_max {
            0.0
        } else if ampf > amp_max {
            1.0
        } else {
            (ampf - ff_max) / (amp_max - ff_max)
        };

        let len = mf.len();
        let dur = len as f64 / 44100.0;

        let mut env = Envelope::fast_release();
        env.amp = amp;
        env.mult((0..len).map(move |ix| {
            ff[ix] * ff_amp + mf[ix] * mf_amp + pp[ix] * pp_amp
            // ff[ix]
        }), dur)
    }
}

