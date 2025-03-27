use std::cmp;
use std::sync::Arc;

use crate::midifile::{MidiEvent, MidiFile};
use crate::synthesizer::Synthesizer;

/// An instance of the MIDI file sequencer.
pub struct MidiFileSequencer {
    synthesizer: Synthesizer,

    midi_file: Option<Arc<MidiFile>>,

    block_wrote: usize,

    current_time: f64,
    msg_index: usize,
}

impl MidiFileSequencer {
    /// Initializes a new instance of the sequencer.
    ///
    /// # Arguments
    ///
    /// * `synthesizer` - The synthesizer to be handled by the sequencer.
    pub fn new(synthesizer: Synthesizer) -> Self {
        Self {
            synthesizer,
            midi_file: None,
            block_wrote: 0,
            current_time: 0.0,
            msg_index: 0,
        }
    }

    /// Plays the MIDI file.
    ///
    /// # Arguments
    ///
    /// * `midi_file` - The MIDI file to be played.
    pub fn play(&mut self, midi_file: &Arc<MidiFile>) {
        self.midi_file = Some(Arc::clone(midi_file));

        self.block_wrote = self.synthesizer.block_size;

        self.current_time = 0.0;
        self.msg_index = 0;

        self.synthesizer.reset()
    }

    /// Stops playing.
    pub fn stop(&mut self) {
        self.midi_file = None;
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
        let midi_file = match self.midi_file.as_ref() {
            Some(value) => value,
            None => return,
        };

        while self.msg_index < midi_file.events.len() {
            let MidiEvent { time, ch, msg } = midi_file.events[self.msg_index];
            if time <= self.current_time {
                self.synthesizer.process_midi_message(ch, msg);
                self.msg_index += 1;
            } else {
                break;
            }
        }
    }

    /// Gets the synthesizer handled by the sequencer.
    pub fn get_synthesizer(&self) -> &Synthesizer {
        &self.synthesizer
    }

    /// Gets the currently playing MIDI file.
    pub fn get_midi_file(&self) -> Option<&MidiFile> {
        match &self.midi_file {
            None => None,
            Some(value) => Some(value),
        }
    }

    /// Gets the current playback position in seconds.
    pub fn get_position(&self) -> f64 {
        self.current_time
    }

    /// Gets a value that indicates whether the current playback position is at the end of the sequence.
    ///
    /// # Remarks
    ///
    /// If the `play` method has not yet been called, this value will be `true`.
    /// This value will never be `true` if loop playback is enabled.
    pub fn end_of_sequence(&self) -> bool {
        match &self.midi_file {
            None => true,
            Some(value) => self.msg_index == value.events.len(),
        }
    }
}
