use music_syn::{
    sample_reader::*,
    playback::*,
    types::*,
    midisyn::*,
    instr::*,
    writer::*,
    geniter::GenIter,
};
use std::env;

fn gen_play(m0: &mut MidiSyn,
            m1: &mut MidiSyn,
            es: &[rimd::TrackEvent],
            out_path: Option<&str>) -> R<()> {
    let ss0 = GenIter(m0.syn_gen(es))
        .into_iter()
        .flat_map(|x| x.into_iter());
    let ss1 = GenIter(m1.syn_gen(es))
        .into_iter()
        .flat_map(|x| x.into_iter());

    let ss = ss0.zip(ss1)
        .flat_map(|(x, y)| vec![x, y])
        .map(|x| x * 0.5);

    if let Some(out_path) = out_path {
        println!("Writing to {}...", out_path);
        save_wav(ss, out_path, 2)?;
    } else {
        // let ss: Vec<f32> = ss.collect();
        println!("Playing...");
        let mut settings = Settings::default();
        settings.channels = 2;
        settings.frames_per_buffer = 640;
        play(&settings, ss.into_iter())?;
    }

    Ok(())
}

fn main() -> R<()> {
    let args: Vec<String> = env::args().collect();
    let mut out_file: Option<&str> = None;
    if args.len() == 2 {
    } else if args.len() == 3 {
        out_file = Some(&args[2]);
    } else {
        println!("Usage: {} $MIDI_IN [$WAV_OUT]", args[0]);
        return Ok(());
    }
    let in_file = &args[1];

    let f = read_midi(in_file)?;

    println!("Loading piano samples, this might take several seconds...");
    let (p0, p1) = Piano::load("samples/normed")?;

    let mut msyn0 = MidiSyn::new(p0);
    let mut msyn1 = MidiSyn::new(p1);
    // XXX: sanity check >0
    msyn0.track_state.div = f.division as usize;
    msyn1.track_state.div = f.division as usize;
    let events = &f.tracks[0].events;
    gen_play(&mut msyn0, &mut msyn1, events, out_file)?;

    Ok(())
}
