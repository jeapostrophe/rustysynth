use crate::array_math::ArrayMath;
use crate::channel::Channel;
use crate::chorus::Chorus;
use crate::loop_mode::LoopMode;
use crate::reverb::Reverb;
use crate::soundfont_math::SoundFontMath;
use crate::synthesizer_settings::{SynthesizerError, SynthesizerSettings};
use crate::voice_collection::VoiceCollection;
use anyhow::Result;
use midly::num::u4;
use std::cmp;

pub trait Sound {
    fn sample_sample_rate(&self) -> i32;
    fn get_sample_start(&self) -> i32;
    fn get_sample_end(&self) -> i32;
    fn get_sample_start_loop(&self) -> i32;
    fn get_sample_end_loop(&self) -> i32;
    fn get_modulation_lfo_to_pitch(&self) -> i32;
    fn get_vibrato_lfo_to_pitch(&self) -> i32;
    fn get_modulation_envelope_to_pitch(&self) -> i32;
    fn get_initial_filter_cutoff_frequency(&self) -> f32;
    fn get_initial_filter_q(&self) -> f32;
    fn get_modulation_lfo_to_filter_cutoff_frequency(&self) -> i32;
    fn get_modulation_envelope_to_filter_cutoff_frequency(&self) -> i32;
    fn get_modulation_lfo_to_volume(&self) -> f32;
    fn get_chorus_effects_send(&self) -> f32;
    fn get_reverb_effects_send(&self) -> f32;
    fn get_pan(&self) -> f32;
    fn get_delay_modulation_lfo(&self) -> f32;
    fn get_frequency_modulation_lfo(&self) -> f32;
    fn get_delay_vibrato_lfo(&self) -> f32;
    fn get_frequency_vibrato_lfo(&self) -> f32;
    fn get_delay_modulation_envelope(&self) -> f32;
    fn get_attack_modulation_envelope(&self) -> f32;
    fn get_hold_modulation_envelope(&self) -> f32;
    fn get_decay_modulation_envelope(&self) -> f32;
    fn get_sustain_modulation_envelope(&self) -> f32;
    fn get_release_modulation_envelope(&self) -> f32;
    fn get_key_number_to_modulation_envelope_hold(&self) -> i32;
    fn get_key_number_to_modulation_envelope_decay(&self) -> i32;
    fn get_delay_volume_envelope(&self) -> f32;
    fn get_attack_volume_envelope(&self) -> f32;
    fn get_hold_volume_envelope(&self) -> f32;
    fn get_decay_volume_envelope(&self) -> f32;
    fn get_sustain_volume_envelope(&self) -> f32;
    fn get_release_volume_envelope(&self) -> f32;
    fn get_key_number_to_volume_envelope_hold(&self) -> i32;
    fn get_key_number_to_volume_envelope_decay(&self) -> i32;
    fn get_initial_attenuation(&self) -> f32;
    fn get_coarse_tune(&self) -> i32;
    fn get_fine_tune(&self) -> i32;
    fn get_sample_modes(&self) -> LoopMode;
    fn get_scale_tuning(&self) -> i32;
    fn get_exclusive_class(&self) -> i32;
    fn get_root_key(&self) -> i32;
}

pub trait SoundSource {
    fn get_regions(
        &self,
        bank_id: i32,
        patch_id: i32,
        key: i32,
        velocity: i32,
    ) -> Result<impl Sound>;
    fn wave_data(&self) -> &Vec<i16>;
}

/// An instance of the SoundFont synthesizer.
#[derive(Debug)]
pub struct Synthesizer<SoundSource> {
    pub(crate) sound_font: SoundSource,
    pub(crate) sample_rate: i32,
    pub(crate) block_size: usize,

    channels: Vec<Channel>,

    voices: VoiceCollection,

    block_left: Vec<f32>,
    block_right: Vec<f32>,

    inverse_block_size: f32,

    block_read: usize,

    master_volume: f32,

    effects: Option<Effects>,
}

/// The percussion channel.
pub const PERCUSSION_CHANNEL: usize = 9;
fn write_block(
    previous_gain: f32,
    current_gain: f32,
    source: &[f32],
    destination: &mut [f32],
    inverse_block_size: f32,
) {
    if SoundFontMath::max(previous_gain, current_gain) < SoundFontMath::NON_AUDIBLE {
        return;
    }

    if (current_gain - previous_gain).abs() < 1.0E-3_f32 {
        ArrayMath::multiply_add(current_gain, source, destination);
    } else {
        let step = inverse_block_size * (current_gain - previous_gain);
        ArrayMath::multiply_add_slope(previous_gain, step, source, destination);
    }
}

impl<Source: SoundSource> Synthesizer<Source> {
    /// Initializes a new synthesizer using a specified SoundFont and settings.
    ///
    /// # Arguments
    ///
    /// * `sound_font` - The SoundFont instance.
    /// * `settings` - The settings for synthesis.
    pub fn new<S>(
        sound_font_pre: S,
        settings: &SynthesizerSettings,
    ) -> Result<Self, SynthesizerError>
    where
        Source: From<S>,
    {
        let sound_font = sound_font_pre.into();
        settings.validate()?;

        let mut channels: Vec<Channel> = Vec::new();
        for i in 0..(u4::max_value().as_int() as usize) {
            channels.push(Channel::new(i == PERCUSSION_CHANNEL));
        }

        let voices = VoiceCollection::new(settings);

        let block_left: Vec<f32> = vec![0_f32; settings.block_size];
        let block_right: Vec<f32> = vec![0_f32; settings.block_size];

        let inverse_block_size = 1_f32 / settings.block_size as f32;

        let block_read = settings.block_size;

        let master_volume = 0.5_f32;

        let effects = if settings.enable_reverb_and_chorus {
            Some(Effects::new(settings))
        } else {
            None
        };

        Ok(Self {
            sound_font,
            sample_rate: settings.sample_rate,
            block_size: settings.block_size,
            channels,
            voices,
            block_left,
            block_right,
            inverse_block_size,
            block_read,
            master_volume,
            effects,
        })
    }

    /// Processes a MIDI message.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to which the message will be sent.
    /// * `command` - The type of the message.
    /// * `data1` - The first data part of the message.
    /// * `data2` - The second data part of the message.
    pub fn process_midi_message<'a>(&mut self, channel: u4, msg: midly::MidiMessage) {
        let channel = channel.as_int() as i32;
        let channel_info = &mut self.channels[channel as usize];

        use midly::MidiMessage;
        match msg {
            MidiMessage::NoteOff { key, .. } => self.note_off(channel, key.as_int().into()),
            MidiMessage::NoteOn { key, vel } => {
                self.note_on(channel, key.as_int().into(), vel.as_int().into())
            }
            MidiMessage::Controller { controller, value } => match controller.as_int() {
                0x00 => channel_info.set_bank(value.as_int().into()),
                0x01 => channel_info.set_modulation_coarse(value.as_int().into()),
                0x21 => channel_info.set_modulation_fine(value.as_int().into()),
                0x06 => channel_info.data_entry_coarse(value.as_int().into()),
                0x26 => channel_info.data_entry_fine(value.as_int().into()),
                0x07 => channel_info.set_volume_coarse(value.as_int().into()),
                0x27 => channel_info.set_volume_fine(value.as_int().into()),
                0x0A => channel_info.set_pan_coarse(value.as_int().into()),
                0x2A => channel_info.set_pan_fine(value.as_int().into()),
                0x0B => channel_info.set_expression_coarse(value.as_int().into()),
                0x2B => channel_info.set_expression_fine(value.as_int().into()),
                0x40 => channel_info.set_hold_pedal(value.as_int().into()),
                0x5B => channel_info.set_reverb_send(value.as_int().into()),
                0x5D => channel_info.set_chorus_send(value.as_int().into()),
                0x63 => channel_info.set_nrpn_coarse(value.as_int().into()),
                0x62 => channel_info.set_nrpn_fine(value.as_int().into()),
                0x65 => channel_info.set_rpn_coarse(value.as_int().into()),
                0x64 => channel_info.set_rpn_fine(value.as_int().into()),
                0x78 => self.note_off_all_channel(channel, true),
                0x79 => self.reset_all_controllers_channel(channel),
                0x7B => self.note_off_all_channel(channel, false),
                _ => (),
            },
            MidiMessage::ProgramChange { program } => {
                channel_info.set_patch(program.as_int().into())
            }
            MidiMessage::PitchBend { bend } => channel_info.set_pitch_bend(bend.0.as_int().into()),
            _ => (),
        }
    }

    /// Stops a note.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel of the note.
    /// * `key` - The key of the note.
    pub fn note_off(&mut self, channel: i32, key: i32) {
        for voice in self.voices.get_active_voices().iter_mut() {
            if voice.channel == channel && voice.key == key {
                voice.end();
            }
        }
    }

    /// Starts a note.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel of the note.
    /// * `key` - The key of the note.
    /// * `velocity` - The velocity of the note.
    pub fn note_on(&mut self, channel: i32, key: i32, velocity: i32) {
        if velocity == 0 {
            self.note_off(channel, key);
            return;
        }

        let channel_info = &self.channels[channel as usize];

        if let Ok(region_pair) = self.sound_font.get_regions(
            channel_info.get_bank_number(),
            channel_info.get_patch_number(),
            key,
            velocity,
        ) {
            if let Some(value) = self.voices.request_new(&region_pair, channel) {
                value.start(&region_pair, channel, key, velocity)
            }
        }
    }

    /// Stops all the notes in the specified channel.
    ///
    /// # Arguments
    ///
    /// * `immediate` - If `true`, notes will stop immediately without the release sound.
    pub fn note_off_all(&mut self, immediate: bool) {
        if immediate {
            self.voices.clear();
        } else {
            for voice in self.voices.get_active_voices().iter_mut() {
                voice.end();
            }
        }
    }

    /// Stops all the notes in the specified channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel in which the notes will be stopped.
    /// * `immediate` - If `true`, notes will stop immediately without the release sound.
    pub fn note_off_all_channel(&mut self, channel: i32, immediate: bool) {
        for voice in self.voices.get_active_voices().iter_mut() {
            if voice.channel == channel {
                if immediate {
                    voice.kill();
                } else {
                    voice.end();
                }
            }
        }
    }

    /// Resets all the controllers.
    pub fn reset_all_controllers(&mut self) {
        for channel in &mut self.channels {
            channel.reset_all_controllers();
        }
    }

    /// Resets all the controllers of the specified channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to be reset.
    pub fn reset_all_controllers_channel(&mut self, channel: i32) {
        self.channels[channel as usize].reset_all_controllers();
    }

    /// Resets the synthesizer.
    pub fn reset(&mut self) {
        self.voices.clear();

        for channel in &mut self.channels {
            channel.reset();
        }

        if let Some(effects) = self.effects.as_mut() {
            effects.reverb.mute();
            effects.chorus.mute();
        }

        self.block_read = self.block_size;
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
        if left.len() != right.len() {
            panic!("The output buffers for the left and right must be the same length.");
        }

        let left_length = left.len();

        let mut wrote = 0;
        while wrote < left_length {
            if self.block_read == self.block_size {
                self.render_block();
                self.block_read = 0;
            }

            let src_rem = self.block_size - self.block_read;
            let dst_rem = left_length - wrote;
            let rem = cmp::min(src_rem, dst_rem);

            for t in 0..rem {
                left[wrote + t] = self.block_left[self.block_read + t];
                right[wrote + t] = self.block_right[self.block_read + t];
            }

            self.block_read += rem;
            wrote += rem;
        }
    }

    fn render_block(&mut self) {
        self.voices
            .process(self.sound_font.wave_data(), &self.channels);

        self.block_left.fill(0_f32);
        self.block_right.fill(0_f32);
        for voice in self.voices.get_active_voices().iter_mut() {
            let previous_gain_left = self.master_volume * voice.previous_mix_gain_left;
            let current_gain_left = self.master_volume * voice.current_mix_gain_left;
            write_block(
                previous_gain_left,
                current_gain_left,
                &voice.block[..],
                &mut self.block_left[..],
                self.inverse_block_size,
            );
            let previous_gain_right = self.master_volume * voice.previous_mix_gain_right;
            let current_gain_right = self.master_volume * voice.current_mix_gain_right;
            write_block(
                previous_gain_right,
                current_gain_right,
                &voice.block[..],
                &mut self.block_right[..],
                self.inverse_block_size,
            );
        }

        if let Some(effects) = self.effects.as_mut() {
            let chorus = &mut effects.chorus;
            let chorus_input_left = &mut effects.chorus_input_left[..];
            let chorus_input_right = &mut effects.chorus_input_right[..];
            let chorus_output_left = &mut effects.chorus_output_left[..];
            let chorus_output_right = &mut effects.chorus_output_right[..];
            chorus_input_left.fill(0_f32);
            chorus_input_right.fill(0_f32);
            for voice in self.voices.get_active_voices().iter_mut() {
                let previous_gain_left = voice.previous_chorus_send * voice.previous_mix_gain_left;
                let current_gain_left = voice.current_chorus_send * voice.current_mix_gain_left;
                write_block(
                    previous_gain_left,
                    current_gain_left,
                    &voice.block[..],
                    chorus_input_left,
                    self.inverse_block_size,
                );
                let previous_gain_right =
                    voice.previous_chorus_send * voice.previous_mix_gain_right;
                let current_gain_right = voice.current_chorus_send * voice.current_mix_gain_right;
                write_block(
                    previous_gain_right,
                    current_gain_right,
                    &voice.block[..],
                    chorus_input_right,
                    self.inverse_block_size,
                );
            }
            chorus.process(
                chorus_input_left,
                chorus_input_right,
                chorus_output_left,
                chorus_output_right,
            );
            ArrayMath::multiply_add(
                self.master_volume,
                chorus_output_left,
                &mut self.block_left[..],
            );
            ArrayMath::multiply_add(
                self.master_volume,
                chorus_output_right,
                &mut self.block_right[..],
            );

            let reverb = &mut effects.reverb;
            let reverb_input = &mut effects.reverb_input[..];
            let reverb_output_left = &mut effects.reverb_output_left[..];
            let reverb_output_right = &mut effects.reverb_output_right[..];
            reverb_input.fill(0_f32);
            for voice in self.voices.get_active_voices().iter_mut() {
                let previous_gain = reverb.get_input_gain()
                    * voice.previous_reverb_send
                    * (voice.previous_mix_gain_left + voice.previous_mix_gain_right);
                let current_gain = reverb.get_input_gain()
                    * voice.current_reverb_send
                    * (voice.current_mix_gain_left + voice.current_mix_gain_right);
                write_block(
                    previous_gain,
                    current_gain,
                    &voice.block[..],
                    &mut reverb_input[..],
                    self.inverse_block_size,
                );
            }

            reverb.process(reverb_input, reverb_output_left, reverb_output_right);
            ArrayMath::multiply_add(
                self.master_volume,
                reverb_output_left,
                &mut self.block_left[..],
            );
            ArrayMath::multiply_add(
                self.master_volume,
                reverb_output_right,
                &mut self.block_right[..],
            );
        }
    }
}

#[derive(Debug)]
struct Effects {
    reverb: Reverb,
    reverb_input: Vec<f32>,
    reverb_output_left: Vec<f32>,
    reverb_output_right: Vec<f32>,

    chorus: Chorus,
    chorus_input_left: Vec<f32>,
    chorus_input_right: Vec<f32>,
    chorus_output_left: Vec<f32>,
    chorus_output_right: Vec<f32>,
}

impl Effects {
    fn new(settings: &SynthesizerSettings) -> Effects {
        Self {
            reverb: Reverb::new(settings.sample_rate),
            reverb_input: vec![0_f32; settings.block_size],
            reverb_output_left: vec![0_f32; settings.block_size],
            reverb_output_right: vec![0_f32; settings.block_size],
            chorus: Chorus::new(settings.sample_rate, 0.002, 0.0019, 0.4),
            chorus_input_left: vec![0_f32; settings.block_size],
            chorus_input_right: vec![0_f32; settings.block_size],
            chorus_output_left: vec![0_f32; settings.block_size],
            chorus_output_right: vec![0_f32; settings.block_size],
        }
    }
}
