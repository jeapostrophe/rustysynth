use crate::midifile::{MidiEvent, MidiFile};
use crate::MidiAdapter;
use rustysynth::{SoundSource, Synthesizer};

/// An instance of the MIDI file sequencer.
pub struct MidiFileSequencer<Source> {
    synthesizer: Synthesizer<Source>,
    midi_file: MidiFile,
    current_time: f64,
    msg_index: usize,
}

impl<Source: SoundSource> MidiFileSequencer<Source> {
    pub fn new(mut synthesizer: Synthesizer<Source>, midi_file: MidiFile) -> Self {
        synthesizer.reset();
        Self {
            synthesizer,
            midi_file,
            current_time: 0.0,
            msg_index: 0,
        }
    }

    pub fn stop(&mut self) {
        self.synthesizer.reset();
    }

    pub fn render(&mut self) -> (f32, f32) {
        self.process_events();
        self.current_time += 1.0 / rustysynth::SAMPLE_RATE as f64;
        self.synthesizer.render()
    }

    fn process_events(&mut self) {
        while self.msg_index < self.midi_file.events.len() {
            let MidiEvent { time, ch, msg } = self.midi_file.events[self.msg_index];
            if time <= self.current_time {
                self.synthesizer.process_midi_message(ch, msg);
                self.msg_index += 1;
            } else {
                break;
            }
        }
    }

    pub fn end_of_sequence(&self) -> bool {
        self.msg_index == self.midi_file.events.len()
    }
}
