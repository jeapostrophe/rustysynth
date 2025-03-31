mod midifile;
mod midifile_sequencer;

pub use self::midifile::MidiFile;
pub use self::midifile_sequencer::MidiFileSequencer;

use midly::{num::u4, MidiMessage};
use rustysynth::{SoundSource, Synthesizer};

pub trait MidiAdapter {
    fn process_midi_message(&mut self, channel: u4, msg: MidiMessage);
}

impl<Source: SoundSource> MidiAdapter for Synthesizer<Source> {
    fn process_midi_message(&mut self, channel: u4, msg: MidiMessage) {
        let channel = channel.as_int();

        use midly::MidiMessage;
        match msg {
            MidiMessage::NoteOff { key, .. } => self.note_off(channel.into(), key.as_int().into()),
            MidiMessage::NoteOn { key, vel } => {
                self.note_on(channel.into(), key.as_int().into(), vel.as_int().into())
            }
            MidiMessage::Controller { controller, value } => match controller.as_int() {
                0x00 => self.set_bank(channel, value.as_int()),
                0x01 => self.set_modulation_coarse(channel, value.as_int()),
                0x21 => self.set_modulation_fine(channel, value.as_int()),
                0x06 => self.data_entry_coarse(channel, value.as_int()),
                0x26 => self.data_entry_fine(channel, value.as_int()),
                0x07 => self.set_volume_coarse(channel, value.as_int()),
                0x27 => self.set_volume_fine(channel, value.as_int()),
                0x0A => self.set_pan_coarse(channel, value.as_int()),
                0x2A => self.set_pan_fine(channel, value.as_int()),
                0x0B => self.set_expression_coarse(channel, value.as_int()),
                0x2B => self.set_expression_fine(channel, value.as_int()),
                0x40 => self.set_hold_pedal(channel, value.as_int()),
                0x5B => self.set_reverb_send(channel, value.as_int()),
                0x5D => self.set_chorus_send(channel, value.as_int()),
                0x63 => self.set_nrpn_coarse(channel, value.as_int()),
                0x62 => self.set_nrpn_fine(channel, value.as_int()),
                0x65 => self.set_rpn_coarse(channel, value.as_int()),
                0x64 => self.set_rpn_fine(channel, value.as_int()),
                0x78 => self.note_off_all_channel(channel.into(), true),
                0x79 => self.reset_all_controllers_channel(channel.into()),
                0x7B => self.note_off_all_channel(channel.into(), false),
                _ => (),
            },
            MidiMessage::ProgramChange { program } => self.set_patch(channel, program.as_int()),
            MidiMessage::PitchBend { bend } => self.set_pitch_bend(channel, bend.0.as_int()),
            _ => (),
        }
    }
}
