#![allow(warnings)]

use music_syn::{
    sample_reader::*,
    playback::*,
    soundprim::*,
    types::*,
    midisyn::*,
    instr::*,
    writer::*,
    geniter::GenIter,
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

fn gen_play(m: &mut MidiSyn, es: &[rimd::TrackEvent]) -> R<()> {
    let ss = GenIter(m.syn_gen(es))
        .into_iter()
        .flat_map(|x| x.into_iter())
        .map(|x| x * 0.2);
    let ss: Vec<f32> = ss.collect();
    play_def(ss.into_iter())?;
    // save_wav(ss, "deb_clai.wav")?;
    Ok(())
}

fn main() -> R<()> {
    // let f = read_midi("midi/mz_545_1_format0.mid")?;
    // let f = read_midi("midi/bach_846_format0.mid")?;
    // let f = read_midi("midi/mz_331_3_format0.mid")?;
    let f = read_midi("midi/chpn_op66_format0.mid")?;
    // let f = read_midi("midi/deb_clai_format0.mid")?;

    let p = Piano::load("samples/normed")?;
    println!("Piano loaded.");

    let mut msyn = MidiSyn::new(p);
    // XXX: sanity check >0
    msyn.track_state.div = f.division as usize;
    let events = &f.tracks[0].events;
    println!("Total events: {}", events.len());
    gen_play(&mut msyn, events)?;

    // let output = msyn.syn(&events[..]);
    // println!("Number of seconds: {}", output.len() / 44100);
    // let ss: Vec<f32> = output.iter().map(|x| x * 0.1).collect();
    // play_def(ss.into_iter())?;
    // save_wav(ss.into_iter(), "deb_clai.wav")?;

    Ok(())
}
