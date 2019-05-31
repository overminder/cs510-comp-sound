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

fn gen_play(m0: &mut MidiSyn,
            m1: &mut MidiSyn,
            es: &[rimd::TrackEvent]) -> R<()> {
    let ss0 = GenIter(m0.syn_gen(es))
        .into_iter()
        .flat_map(|x| x.into_iter());
    let ss1 = GenIter(m1.syn_gen(es))
        .into_iter()
        .flat_map(|x| x.into_iter());

    println!("Synthesizing...");
    let ss = ss0.zip(ss1)
        .flat_map(|(x, y)| vec![x, y])
        .map(|x| x * 0.5);
    // let ss: Vec<f32> = ss.collect();
    println!("Done, playing...");
    let mut settings = Settings::default();
    settings.channels = 2;
    // play(&settings, ss.into_iter())?;
    save_wav(ss, "mz_545.wav", 2)?;
    Ok(())
}

fn main() -> R<()> {
    let f = read_midi("midi/mz_545_1_format0.mid")?;
    // let f = read_midi("midi/mz_545_3_format0.mid")?;
    // let f = read_midi("midi/bach_846_format0.mid")?;
    // let f = read_midi("midi/mz_331_3_format0.mid")?;
    // let f = read_midi("midi/chpn_op66_format0.mid")?;
    // let f = read_midi("midi/deb_clai_format0.mid")?;

    let (p0, p1) = Piano::load("samples/normed")?;
    println!("Piano loaded.");

    let mut msyn0 = MidiSyn::new(p0);
    let mut msyn1 = MidiSyn::new(p1);
    // XXX: sanity check >0
    msyn0.track_state.div = f.division as usize;
    msyn1.track_state.div = f.division as usize;
    let events = &f.tracks[0].events;
    println!("Total events: {}", events.len());
    gen_play(&mut msyn0, &mut msyn1, events)?;

    // let output = msyn.syn(&events[..]);
    // println!("Number of seconds: {}", output.len() / 44100);
    // let ss: Vec<f32> = output.iter().map(|x| x * 0.1).collect();
    // play_def(ss.into_iter())?;
    // save_wav(ss.into_iter(), "deb_clai.wav")?;

    Ok(())
}
