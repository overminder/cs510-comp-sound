# Sample Based Synthesis

https://courses.cs.washington.edu/courses/cse490s/11au/Readings/SynthesisChapt4a.pdf
shows some ways for looping

# MIDI

- http://www.indiana.edu/~emusic/etext/MIDI/chapter3_MIDI4.shtml
- http://midi.teragonaudio.com/tech/midispec/noteon.htm
- https://web.archive.org/web/20150217154504/http://cs.fit.edu/~ryan/cse4051/projects/midi/midi.html#meta_event
- https://www.nyu.edu/classes/bello/FMT_files/9_MIDI_code.pdf
- http://www.tonalsoft.com/pub/pitch-bend/pitch.2005-08-31.17-00.aspx
- http://www.music-software-development.com/midi-tutorial.html

## Impl

### Deciding the timing

- SMF.division: ticks per beat
- TrackEvent.vtime: ticks after previous event
- Meta.TempoSetting: real time (in micros) per beat. (i.e. 60 / such = BPM)

### Deciding the instrument/pitch

- Meta.KeySig: sharp/flat, major/minor
- Meta.ProgChange: [instrument-preset] on [channel]
  + See https://www.earmaster.com/wiki/music-technology/what-is-midi.html
    for list of presets. Minus 1 for zero-based indexing.
- Meta.CtrlChange: [ctrl] [option] on [channel]
  + See http://www.indiana.edu/~emusic/etext/MIDI/chapter3_MIDI6.shtml
  + ctrl=64: damper pedal (option<64: off, >=64: on)

### Playing the note

- Midi.NoteOn [key] [velo]
  + key: 60=C4 (261.63 Hz)
  + velo: pp=32 mf=80 ff=112
  + velo=0 also represents NoteOff
- Midi.NoteOff [key] [velo]

## Nomenclature

- Beat: a 1/4 note.
