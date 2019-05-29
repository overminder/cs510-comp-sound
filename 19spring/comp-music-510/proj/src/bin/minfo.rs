#![allow(warnings)]

use music_syn::types::*;
use rimd::*;

fn is_control(f: &TrackEvent) -> bool {
    // Found: 7 (volume), 10 (pan), 91 (depth effect), 64 (damper pedal)
    match &f.event {
        Event::Midi(m) if m.status() == Status::ControlChange => true,
        _ => false,
    }
}

fn debug_track(f: &SMF) {
    println!("fmt = {}, #tr = {}, div = {}",
             f.format, f.tracks.len(), f.division);
    for (i, t) in f.tracks.iter().enumerate() {
        println!("Track {}: {}", i, t);
        for e in t.events.iter().filter(|x| is_control(x)).take(15) {
            println!("  {}", e);
        }
    }
}

fn main() -> R<()> {
    let f = SMF::from_file("midi/bach_846_format0.mid".as_ref())?;
    debug_track(&f);
    Ok(())
}

