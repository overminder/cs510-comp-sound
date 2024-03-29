use crate::soundprim::*;
use crate::types::*;
use crate::instr::*;

use std::ops::Generator;
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

type NoteMap = HashMap<u8, Box<dyn Sound>>;
type NoteVec = Vec<Box<dyn Sound>>;

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

fn elapse_vec(ns: &mut NoteVec, out: &mut [f32]) {
    let len = out.len();
    let mut t = NoteVec::new();
    mem::swap(ns, &mut t);
    
    // For each sample,
    for mut s in t {
        // Take len samples
        let mut empty = false;
        for i in 0..len {
            if let Some(v) = s.next() {
                out[i] += v;
            } else {
                empty = true;
                break;
            }
        }
        if !empty {
            ns.push(s);
        }
    }
}

fn elapse_map(ns: &mut NoteMap, out: &mut [f32]) {
    let len = out.len();
    let mut t = NoteMap::new();
    mem::swap(ns, &mut t);

    for (key, mut s) in t {
        // Take len samples
        let mut empty = false;
        for i in 0..len {
            if let Some(v) = s.next() {
                out[i] += v;
            } else {
                empty = true;
                break;
            }
        }
        if !empty {
            ns.insert(key, s);
        }
    }
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

    pub fn syn_gen<'a>(&'a mut self, track: &'a [TrackEvent])
        -> impl Generator<Yield=Vec<f32>> + Unpin + 'a {
        self.output.clear();
        move || {
            for te in track {
                let mut v = vec![];
                self.elapse_ticks(te.vtime);
                if te.vtime != 0 {
                    mem::swap(&mut v, &mut self.output);
                    yield v;
                }
                match &te.event {
                    Event::Midi(msg) =>
                        self.do_midi(msg),
                    Event::Meta(meta) =>
                        self.do_meta(meta),
                }
            }
        }
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
        let nsamples = nsamples as usize;

        // For each sample,
        let start = self.output.len();
        self.output.resize(start + nsamples, 0.0);
        let dst = &mut self.output[start..];
        // let mut output = vec![0.0; nsamples];
        // Advance currently ongoing sounds by nsamples.
        elapse_map(&mut self.sounds, dst);
        elapse_vec(&mut self.dampered_sounds, dst);
        elapse_vec(&mut self.released_sounds, dst);
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
                let _sharp = meta.data[0] as i8;
                let _major = meta.data[1] == 0;
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
        if self.sounds.contains_key(&key) {
            // Assume that the intention is to re-press this key.
            self.do_note_off(key);
        }

        let key_wrt_c4 = (key as i32) - 60;
        let duration = 1.0;
        let amp = (velo as f64) / 128.0;

        let ss: Box<dyn Sound> = match self.track_state.instrument {
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

        self.sounds.insert(key, ss);
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
        let instr = if preset <= 7 {
            // Generic piano for 0-7
            Instrument::Piano
        } else {
            println!("Unsupported ProgChange(preset={})", preset);
            Instrument::NoImpl
        };
        self.track_state.instrument = instr;
    }

    fn do_ctrl_change(&mut self, ctrl: u8, option: u8) {
        if ctrl == 64 {
            let on = option >= 64;
            if self.track_state.damper_pedal != on {
                // Releasing damper pedal: apply to existing sounds.
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

