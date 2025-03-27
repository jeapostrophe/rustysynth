use crate::midifile::{MidiEvent, MidiFile};
use crate::synthesizer::Synthesizer;
use std::cmp;

/// An instance of the MIDI file sequencer.
pub struct MidiFileSequencer {
    synthesizer: Synthesizer,

    midi_file: MidiFile,

    block_wrote: usize,

    current_time: f64,
    msg_index: usize,
}

impl MidiFileSequencer {
    pub fn new(mut synthesizer: Synthesizer, midi_file: MidiFile) -> Self {
        synthesizer.reset();
        let block_wrote = synthesizer.block_size;
        Self {
            synthesizer,
            midi_file,
            block_wrote,
            current_time: 0.0,
            msg_index: 0,
        }
    }

    pub fn stop(&mut self) {
        self.synthesizer.reset();
    }

    /// Renders the waveform.
    ///
    /// # Arguments
    ///
    /// * `left` - The buffer of the left channel to store the rendered waveform.
    /// * `right` - The buffer of the right channel to store the rendered waveform.
    ///
    /// # Remarks
    ///
    /// The output buffers for the left and right must be the same length.
    pub fn render(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert!(
            left.len() == right.len(),
            "The output buffers for the left and right must be the same length."
        );

        let left_length = left.len();
        let mut wrote: usize = 0;
        while wrote < left_length {
            if self.block_wrote == self.synthesizer.block_size {
                self.process_events();
                self.block_wrote = 0;
                self.current_time +=
                    self.synthesizer.block_size as f64 / self.synthesizer.sample_rate as f64;
            }

            let src_rem = self.synthesizer.block_size - self.block_wrote;
            let dst_rem = left_length - wrote;
            let rem = cmp::min(src_rem, dst_rem);

            self.synthesizer.render(
                &mut left[wrote..wrote + rem],
                &mut right[wrote..wrote + rem],
            );

            self.block_wrote += rem;
            wrote += rem;
        }
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
