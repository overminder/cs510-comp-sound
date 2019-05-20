#![allow(warnings)]

use music_syn::{
    sample_reader::*,
    playback::*,
    soundprim::*,
    types::*,
    midisyn::*,
};

fn i16_to_f32_norm(x: i16) -> f32 {
    (x as f32 / i16::max_value() as f32) * 6.
}

fn merge_stereo(v: &[i16]) -> Vec<f32> {
    stereo_channels_iter(&v)
        .map(|(l, r)| (i16_to_f32_norm(l) + i16_to_f32_norm(r)) / 2.0)
        .collect()
}

fn debug_track(f: &rimd::SMF) {
    println!("fmt = {}, #tr = {}, div = {}",
             f.format, f.tracks.len(), f.division);
    for t in &f.tracks {
        println!("{}", t);
        for e in &t.events[..20] {
            println!("{}", e);
        }
    }
}

fn main() -> R<()> {
    let f = read_midi("midi/mz_545_1_format0.mid")?;

    let mut msyn = MidiSyn::new();
    // XXX: sanity check >0
    msyn.track_state.div = f.division as usize;
    let events = &f.tracks[0].events;
    println!("Total events: {}", events.len());
    let output = msyn.syn(&events[..1000]);
    println!("Number of seconds: {}", output.len() / 44100);
    let ss = output.to_vec();
    play_def(ss.into_iter())?;

    Ok(())
}
