use crate::soundprim::*;
use crate::types::*;
use crate::instr::*;

use std::mem;
use std::collections::HashMap;
use rimd::{
    TrackEvent,
    Event,
    MidiMessage,
    Status as MidiStatus,
    MetaEvent,
    MetaCommand,
};

type NoteMap = HashMap<u8, Box<Sound>>;
type NoteVec = Vec<Box<Sound>>;

pub struct MidiSyn {
    pub sample_rate: f64,
    pub track_state: TrackState,

    // Stores the currently pressed notes
    sounds: NoteMap,

    // Stores the released-while-dampered notes. When
    // the pedal is released, these sounds are moved to released_sounds.
    dampered_sounds: NoteVec,

    // Stores the released notes.
    released_sounds: NoteVec,

    // Stores the fraction part of the sample index.
    sample_ix: f64,

    output: Vec<f32>,

    // Piano syn
    piano: Piano,
}

#[derive(Copy, Clone)]
enum Instrument {
    Piano,
    NoImpl,
}

fn elapse_vec(ns: &mut NoteVec) -> f32 {
    // Advance currently ongoing sounds by a sample.
    let mut t = NoteVec::new();
    mem::swap(ns, &mut t);
    let mut vs: f32 = 0.0;
    for mut s in t {
        if let Some(v) = s.next() {
            ns.push(s);
            vs += v;
        }
    }
    vs
}

fn elapse_map(ns: &mut NoteMap) -> f32 {
    let mut t = NoteMap::new();
    mem::swap(ns, &mut t);
    let mut vs: f32 = 0.0;
    for (key, mut s) in t {
        if let Some(v) = s.next() {
            ns.insert(key, s);
            vs += v;
        }
    }
    vs
}

impl MidiSyn {
    pub fn new(p: Piano) -> Self {
        Self {
            sample_rate: 44100.0,
            track_state: TrackState::new(),
            sounds: NoteMap::new(),
            dampered_sounds: NoteVec::new(),
            released_sounds: NoteVec::new(),
            sample_ix: 0.0,
            output: vec![],
            piano: p,
        }
    }

    pub fn syn(&mut self, track: &[TrackEvent]) -> &[f32] {
        for te in track {
            self.elapse_ticks(te.vtime);
            match &te.event {
                Event::Midi(msg) =>
                    self.do_midi(msg),
                Event::Meta(meta) =>
                    self.do_meta(meta),
            }
        }
        &self.output
    }

    fn samples_in_tick(&self, ticks: u64) -> f64 {
        let samples_per_tick = self.sample_rate
            * (self.track_state.tempo as f64 / 1_000_000.0) 
            / self.track_state.div as f64;
        ticks as f64 * samples_per_tick
    }

    fn elapse_ticks(&mut self, vt: u64) {
        if vt == 0 {
            return;
        }

        // Precalc these?
        let nsamples = self.sample_ix + self.samples_in_tick(vt);
        self.sample_ix = nsamples % 1.0;
        let nsamples = nsamples as i64;

        // TODO: Could swap these two loops.

        // For each sample,
        for _ in 0..nsamples {
            // Advance currently ongoing sounds by a sample.
            let mut vs = 0.0;
            vs += elapse_map(&mut self.sounds);
            vs += elapse_vec(&mut self.dampered_sounds);
            vs += elapse_vec(&mut self.released_sounds);
            self.output.push(vs);
        }
    }

    fn do_midi(&mut self, msg: &MidiMessage) {
        use self::MidiStatus::*;

        match msg.status() {
            NoteOn => self.do_note_on(msg.data[1], msg.data[2]),
            NoteOff => self.do_note_off(msg.data[1]),
            ProgramChange => self.do_prog_change(msg.data[1]),
            ControlChange => self.do_ctrl_change(msg.data[1], msg.data[2]),
            _ => {},
        }
    }

    fn do_meta(&mut self, meta: &MetaEvent) {
        use self::MetaCommand::*;

        match meta.command {
            TempoSetting => {
                let tempo = meta.data_as_u64(3);
                self.track_state.tempo = tempo as usize;
            }
            KeySignature => {
                let sharp = meta.data[0] as i8;
                let major = meta.data[1] == 0;
                // TODO
            }
            EndOfTrack => {
                // TODO
            }
            _ => {},
        }
    }

    fn do_note_on(&mut self, key: u8, velo: u8) {
        if velo == 0 {
            return self.do_note_off(key)
        }

        let key_wrt_c4 = (key as i32) - 60;
        let duration = 1.0;
        let amp = (velo as f64) / 128.0;

        let ss: Box<Sound> = match self.track_state.instrument {
            Instrument::Piano => {
                Box::new(self.piano.syn(key_wrt_c4, amp))
            }
            _ => {
                let synthesizer = Sine {
                    key: key_wrt_c4,
                    duration,
                    amp,
                    sample_rate: self.sample_rate,
                };
                Box::new(synthesizer.syn())
            }
        };

        if self.sounds.contains_key(&key) {
            panic!("Pressing the same key again: {}", key);
        } else {
            self.sounds.insert(key, Box::new(ss));
        }
    }

    fn do_note_off(&mut self, key: u8) {
        if let Some(ss) = self.sounds.remove(&key) {
            if self.track_state.damper_pedal {
                // Move to the dampered sounds.
                self.dampered_sounds.push(ss);
            } else {
                let env = Envelope::just_release();
                self.released_sounds.push(Box::new(env.mult(ss, 0.1)));
            }
        }
    }

    fn do_prog_change(&mut self, preset: u8) {
        let instr = if preset == 0 {
            Instrument::Piano
        } else {
            Instrument::NoImpl
        };
        self.track_state.instrument = instr;
    }

    fn do_ctrl_change(&mut self, ctrl: u8, option: u8) {
        if ctrl == 64 {
            let on = option >= 64;
            if self.track_state.damper_pedal != on {
                // Releaseing damper pedal: apply to existing sounds.
                if !on {
                    let mut ss = vec![];
                    mem::swap(&mut self.dampered_sounds, &mut ss);
                    let env = Envelope::just_release();
                    for s in ss {
                        self.released_sounds.push(Box::new(env.mult(s, 0.1)));
                    }
                }
            }
            self.track_state.damper_pedal = on;
        }
    }
}

fn freq_wrt_c4(key: i32) -> f64 {
    let half_step = 1.0595_f64;
    261.63 * half_step.powi(key)
}

pub struct TrackState {
    // Tick per beat
    pub div: usize,

    // Micros per beat
    tempo: usize,

    instrument: Instrument,

    damper_pedal: bool,
}

impl TrackState {
    fn new() -> Self {
        Self {
            div: 480,
            tempo: 434_000,
            instrument: Instrument::Piano,
            damper_pedal: false,
        }
    }
}

