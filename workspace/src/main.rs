use rustysynth::Synthesizer;
use rustysynth_midi::{MidiFile, MidiFileSequencer};
use rustysynth_soundfont::{SoundFont, SoundFontProc};
use std::fs::File;
use std::io::Write;

fn main() {
    simple_chord();
    flourish();
}

fn simple_chord() {
    // Load the SoundFont.
    let mut sf2 = File::open("TimGM6mb.sf2").unwrap();
    let sound_font = SoundFont::new(&mut sf2).unwrap();

    // Create the synthesizer.
    let mut synthesizer: Synthesizer<SoundFontProc> = Synthesizer::new(sound_font);

    // Play some notes (middle C, E, G).
    synthesizer.note_on(0, 60, 100);
    synthesizer.note_on(0, 64, 100);
    synthesizer.note_on(0, 67, 100);

    // The output buffer (3 seconds).
    let sample_count = (3 * rustysynth::SAMPLE_RATE) as usize;
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // Render the waveform.
    for i in 0..sample_count {
        let (left_sample, right_sample) = synthesizer.render();
        left[i] = left_sample;
        right[i] = right_sample;
    }

    // Write the waveform to the file.
    write_pcm(&left[..], &right[..], "simple_chord.pcm");
}

fn flourish() {
    // Load the SoundFont.
    let mut sf2 = File::open("TimGM6mb.sf2").unwrap();
    let sound_font = SoundFont::new(&mut sf2).unwrap();

    // Load the MIDI file.
    let mut mid = File::open("flourish.mid").unwrap();
    let midi_file = MidiFile::new(&mut mid).unwrap();

    // Create the MIDI file sequencer.
    let synthesizer: Synthesizer<SoundFontProc> = Synthesizer::new(sound_font);
    let sample_count = (rustysynth::SAMPLE_RATE as f64 * midi_file.get_length()) as usize;
    let mut sequencer = MidiFileSequencer::new(synthesizer, midi_file);

    // The output buffer.
    let mut left: Vec<f32> = vec![0_f32; sample_count];
    let mut right: Vec<f32> = vec![0_f32; sample_count];

    // Render the waveform.
    for i in 0..sample_count {
        let (left_sample, right_sample) = sequencer.render();
        left[i] = left_sample;
        right[i] = right_sample;
    }

    // Write the waveform to the file.
    write_pcm(&left[..], &right[..], "flourish.pcm");
}

fn write_pcm(left: &[f32], right: &[f32], path: &str) {
    let mut max: f32 = 0_f32;
    for t in 0..left.len() {
        if left[t].abs() > max {
            max = left[t].abs();
        }
        if right[t].abs() > max {
            max = right[t].abs();
        }
    }
    let a = 0.99_f32 / max;

    let mut buf: Vec<u8> = vec![0; 4 * left.len()];
    for t in 0..left.len() {
        let left_i16 = (a * left[t] * 32768_f32) as i16;
        let right_i16 = (a * right[t] * 32768_f32) as i16;

        let offset = 4 * t;
        buf[offset] = left_i16 as u8;
        buf[offset + 1] = (left_i16 >> 8) as u8;
        buf[offset + 2] = right_i16 as u8;
        buf[offset + 3] = (right_i16 >> 8) as u8;
    }

    let mut pcm = File::create(path).unwrap();
    pcm.write_all(&buf[..]).unwrap();
}
