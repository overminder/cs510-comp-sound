use crate::soundprim::*;
use crate::types::*;

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

pub struct MidiSyn {
    pub sample_rate: f64,
    pub track_state: TrackState,
    sounds: HashMap<u8, Box<Sound>>,

    // Stores the fraction part of the sample index.
    sample_ix: f64,

    output: Vec<f32>,
}

#[derive(Copy, Clone)]
enum Instrument {
    Piano,
    NoImpl,
}

impl MidiSyn {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            track_state: TrackState::new(),
            sounds: HashMap::new(),
            sample_ix: 0.0,
            output: vec![],
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
            let mut t = HashMap::new();
            mem::swap(&mut self.sounds, &mut t);
            let mut vs: f32 = 0.0;
            for (key, mut s) in t {
                if let Some(v) = s.next() {
                    self.sounds.insert(key, s);
                    vs += v;
                }
            }
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
        let duration = 2.0;
        let mut env = Envelope::default();
        let ss = sinewave(
            freq_wrt_c4(key_wrt_c4),
            (self.sample_rate * duration) as usize,
            self.sample_rate);
        let ss = env.mult(ss, duration);
        if let Some(orig_ss) = self.sounds.remove(&key) {
            // If that key is already pressed, merge them.
            self.sounds.insert(key, Box::new(superpos(ss, orig_ss)));
        } else {
            self.sounds.insert(key, Box::new(ss));
        }
    }

    fn do_note_off(&mut self, key: u8) {
        if let Some(ss) = self.sounds.remove(&key) {
            let env = Envelope::just_release();
            self.sounds.insert(key, Box::new(env.mult(ss, 0.1)));
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
            self.track_state.damper_pedal = on;
            // TODO: Also control the existing sounds... How should
            // the data flow?
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

