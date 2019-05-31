use std::f32::consts::PI;
use itertools::Itertools;
use itertools::EitherOrBoth::{Both, Left, Right};
use crate::types::*;

pub fn mult(x: impl Sound, y: impl Sound) -> impl Sound {
    x.zip(y).map(|(x, y)| x * y)
}

pub fn sinewave(freq: f64, ticks: usize, sample_rate: f64) -> impl Sound {
    let step = (freq / sample_rate) as f32 * 2.0 * PI;
    (0..ticks).map(move |t| {
        (t as f32 * step).sin()
    })
}

pub struct Envelope {
    pub attack: f64,
    pub attack_plier: f64,
    pub decay: f64,
    pub decay_plier: f64,
    pub sustain: f64,
    pub sustain_plier: f64,
    pub release: f64,
    pub sample_rate: f64,
    pub amp: f64,
}

impl Envelope {
    pub fn default() -> Self {
        Self {
            attack: 0.1,
            attack_plier: 1.2,
            decay: 0.05,
            decay_plier: 1.0,
            sustain: 0.7,
            sustain_plier: 0.7,
            release: 0.15,
            amp: 1.0,
            sample_rate: SAMPLE_RATE,
        }
    }

    pub fn just_release() -> Self {
        let mut s = Self::default();
        s.attack = 0.0;
        s.decay = 0.0;
        s.sustain = 0.0;
        s.sustain_plier = 1.0;
        s.release = 1.0;
        s
    }

    pub fn fast_release() -> Self {
        let mut s = Self::default();
        s.attack = 0.01;
        s.attack_plier = 1.0;
        s.decay = 0.0;
        s.decay_plier = 1.0;
        s.sustain = 0.9;
        s.sustain_plier = 1.0;
        s.release = 0.09;
        s
    }

    pub fn mult(&self, s: impl Sound, duration: f64) -> impl Sound {
        mult(self.make(duration), s)
    }

    pub fn make(&self, duration: f64) -> impl Sound {
        let real_amp = self.amp;

        let attack = duration * self.attack;
        let decay = duration * self.decay;
        let sustain = duration * self.sustain;
        let release = duration * self.release;
        interpolate_to(0., self.attack_plier * real_amp, attack, self.sample_rate)
            .chain(interpolate_to(self.attack_plier * real_amp,
                                  self.decay_plier * real_amp, decay,
                                  self.sample_rate))
            .chain(interpolate_to(self.decay_plier * real_amp,
                                  self.sustain_plier * real_amp, sustain,
                                  self.sample_rate))
            .chain(interpolate_to(self.sustain_plier * real_amp,
                                  0., release,
                                  self.sample_rate))
    }
}

fn interpolate_to(y0: f64, y1: f64, t: f64,
                  sample_rate: f64) -> impl Sound {
    let ticks = (t * sample_rate) as usize;
    let dy = y1 - y0;
    (0..ticks).map(move |t| {
        (y0 + (t as f64 / ticks as f64) * dy) as f32
    })
}

pub fn superpos(x: impl Sound, y: impl Sound) -> impl Sound {
    x.zip_longest(y)
     .map(|xy| {
         match xy {
             Left(x) => x,
             Right(y) => y,
             Both(x, y) => x + y,
         }
     })
}
