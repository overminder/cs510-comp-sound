use crate::types::Sound;
use crate::soundprim::{
    Envelope,
    sinewave,
};

fn freq_wrt_c4(key: i32) -> f64 {
    let half_step = 1.0595_f64;
    261.63 * half_step.powi(key)
}

pub struct Sine {
    // Number of semitones wrt C4.
    pub key: i32,
    pub duration: f64,
    pub amp: f64,
    pub sample_rate: f64,
}

impl Sine {
    pub fn syn(&self) -> impl Sound {
        let mut env = Envelope::default();
        // Looks more like piano.
        env.attack = 0.03;
        env.decay = 0.04;
        env.sustain = 0.2;
        env.release = 0.73;

        env.attack_plier = 2.0;
        env.decay_plier = 1.0;
        env.sustain_plier = 0.7;

        env.amp = self.amp;
        let ss = sinewave(
            freq_wrt_c4(self.key),
            (self.sample_rate * self.duration) as usize,
            self.sample_rate);
        env.mult(ss, self.duration)
    }

}

