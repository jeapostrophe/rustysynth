use crate::midifile::{MidiEvent, MidiFile};
use crate::MidiAdapter;
use rustysynth::{SoundSource, Synthesizer};
use std::cmp;

/// An instance of the MIDI file sequencer.
pub struct MidiFileSequencer<Source> {
    synthesizer: Synthesizer<Source>,

    midi_file: MidiFile,

    block_wrote: usize,

    current_time: f64,
    msg_index: usize,
}

impl<Source: SoundSource> MidiFileSequencer<Source> {
    pub fn new(mut synthesizer: Synthesizer<Source>, midi_file: MidiFile) -> Self {
        synthesizer.reset();
        let block_wrote = rustysynth::BLOCK_SIZE;
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
        let block_size = rustysynth::BLOCK_SIZE;
        let mut wrote: usize = 0;
        while wrote < left_length {
            if self.block_wrote == block_size {
                self.process_events();
                self.block_wrote = 0;
                self.current_time += block_size as f64 / rustysynth::SAMPLE_RATE as f64;
            }

            let src_rem = block_size - self.block_wrote;
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
