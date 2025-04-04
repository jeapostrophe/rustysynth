use crate::channel::Channel;
use crate::chorus::Chorus;
use crate::reverb::Reverb;
use crate::voice_collection::VoiceCollection;
use crate::Block;
use crate::LoopMode;
use anyhow::Result;
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

#[derive(Debug)]
pub struct Synthesizer<Source> {
    pub(crate) sound_font: Source,
    channels: Vec<Channel>,

    voices: VoiceCollection,

    block_left: Block<f32>,
    block_right: Block<f32>,

    inverse_block_size: f32,

    block_read: usize,

    master_volume: f32,

    effects: Effects,
}

pub const CHANNELS: usize = 16;
pub const PERCUSSION_CHANNEL: usize = 9;

// XXX replace these functions with some SIMD operations
mod array_math {
    use crate::soundfont_math::*;

    pub fn multiply_add(a: f32, x: &[f32], destination: &mut [f32]) {
        for (x, destination) in x.iter().zip(destination.iter_mut()) {
            *destination += a * *x;
        }
    }

    pub fn multiply_add_slope(a: f32, step: f32, x: &[f32], destination: &mut [f32]) {
        let mut a = a;
        for (x, destination) in x.iter().zip(destination.iter_mut()) {
            *destination += a * *x;
            a += step;
        }
    }

    pub fn write_block(
        previous_gain: f32,
        current_gain: f32,
        source: &[f32],
        destination: &mut [f32],
        inverse_block_size: f32,
    ) {
        if previous_gain.max(current_gain) < NON_AUDIBLE {
            return;
        }

        if (current_gain - previous_gain).abs() < 1.0E-3_f32 {
            multiply_add(current_gain, source, destination);
        } else {
            let step = inverse_block_size * (current_gain - previous_gain);
            multiply_add_slope(previous_gain, step, source, destination);
        }
    }
}

macro_rules! set_channel {
    ($synth_fun:ident) => {
        set_channel!($synth_fun, u8);
    };
    ($synth_fun:ident, $value_ty:ident) => {
        pub fn $synth_fun(&mut self, channel: u8, value: $value_ty) {
            self.channels[channel as usize].$synth_fun(value.into());
        }
    };
}

impl<Source: SoundSource> Synthesizer<Source> {
    pub fn new<S>(sound_font_pre: S) -> Result<Self>
    where
        Source: From<S>,
    {
        let sound_font = sound_font_pre.into();

        let mut channels: Vec<Channel> = Vec::new();
        for i in 0..CHANNELS {
            channels.push(Channel::new(i == PERCUSSION_CHANNEL));
        }

        let voices = VoiceCollection::new();

        let block_left = [0_f32; crate::BLOCK_SIZE];
        let block_right = [0_f32; crate::BLOCK_SIZE];

        let inverse_block_size = 1_f32 / crate::BLOCK_SIZE as f32;

        let block_read = crate::BLOCK_SIZE;

        let master_volume = 0.5_f32;

        let effects = Effects::default();

        Ok(Self {
            sound_font,
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

    set_channel!(set_bank);
    set_channel!(set_modulation_coarse);
    set_channel!(set_modulation_fine);
    set_channel!(data_entry_coarse);
    set_channel!(data_entry_fine);
    set_channel!(set_volume_coarse);
    set_channel!(set_volume_fine);
    set_channel!(set_pan_coarse);
    set_channel!(set_pan_fine);
    set_channel!(set_expression_coarse);
    set_channel!(set_expression_fine);
    set_channel!(set_hold_pedal);
    set_channel!(set_reverb_send);
    set_channel!(set_chorus_send);
    set_channel!(set_nrpn_coarse);
    set_channel!(set_nrpn_fine);
    set_channel!(set_rpn_coarse);
    set_channel!(set_rpn_fine);
    set_channel!(set_patch);
    set_channel!(set_pitch_bend, u16);

    pub fn note_off(&mut self, channel: i32, key: i32) {
        for voice in self.voices.get_active_voices().iter_mut() {
            if voice.channel == channel && voice.key == key {
                voice.end();
            }
        }
    }

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
            let value = self.voices.request_new();
            value.start(&region_pair, channel, key, velocity)
        }
    }

    pub fn note_off_all(&mut self, immediate: bool) {
        if immediate {
            self.voices.clear();
        } else {
            for voice in self.voices.get_active_voices().iter_mut() {
                voice.end();
            }
        }
    }

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

    pub fn reset_all_controllers(&mut self) {
        for channel in &mut self.channels {
            channel.reset_all_controllers();
        }
    }

    pub fn reset_all_controllers_channel(&mut self, channel: i32) {
        self.channels[channel as usize].reset_all_controllers();
    }

    pub fn reset(&mut self) {
        self.voices.clear();

        for channel in &mut self.channels {
            channel.reset();
        }

        self.effects.reverb.mute();
        self.effects.chorus.mute();

        self.block_read = crate::BLOCK_SIZE;
    }

    pub fn render(&mut self, left: &mut [f32], right: &mut [f32]) {
        if left.len() != right.len() {
            panic!("The output buffers for the left and right must be the same length.");
        }

        let left_length = left.len();

        let mut wrote = 0;
        while wrote < left_length {
            if self.block_read == crate::BLOCK_SIZE {
                self.render_block();
                self.block_read = 0;
            }

            let src_rem = crate::BLOCK_SIZE - self.block_read;
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
            array_math::write_block(
                previous_gain_left,
                current_gain_left,
                &voice.block[..],
                &mut self.block_left[..],
                self.inverse_block_size,
            );
            let previous_gain_right = self.master_volume * voice.previous_mix_gain_right;
            let current_gain_right = self.master_volume * voice.current_mix_gain_right;
            array_math::write_block(
                previous_gain_right,
                current_gain_right,
                &voice.block[..],
                &mut self.block_right[..],
                self.inverse_block_size,
            );
        }

        {
            let effects = &mut self.effects;
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
                array_math::write_block(
                    previous_gain_left,
                    current_gain_left,
                    &voice.block[..],
                    chorus_input_left,
                    self.inverse_block_size,
                );
                let previous_gain_right =
                    voice.previous_chorus_send * voice.previous_mix_gain_right;
                let current_gain_right = voice.current_chorus_send * voice.current_mix_gain_right;
                array_math::write_block(
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
            array_math::multiply_add(
                self.master_volume,
                chorus_output_left,
                &mut self.block_left[..],
            );
            array_math::multiply_add(
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
                array_math::write_block(
                    previous_gain,
                    current_gain,
                    &voice.block[..],
                    &mut reverb_input[..],
                    self.inverse_block_size,
                );
            }

            reverb.process(reverb_input, reverb_output_left, reverb_output_right);
            array_math::multiply_add(
                self.master_volume,
                reverb_output_left,
                &mut self.block_left[..],
            );
            array_math::multiply_add(
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
    reverb_input: Block<f32>,
    reverb_output_left: Block<f32>,
    reverb_output_right: Block<f32>,

    chorus: Chorus,
    chorus_input_left: Block<f32>,
    chorus_input_right: Block<f32>,
    chorus_output_left: Block<f32>,
    chorus_output_right: Block<f32>,
}

impl Default for Effects {
    fn default() -> Effects {
        Self {
            reverb: Reverb::default(),
            reverb_input: [0_f32; crate::BLOCK_SIZE],
            reverb_output_left: [0_f32; crate::BLOCK_SIZE],
            reverb_output_right: [0_f32; crate::BLOCK_SIZE],
            chorus: Chorus::default(),
            chorus_input_left: [0_f32; crate::BLOCK_SIZE],
            chorus_input_right: [0_f32; crate::BLOCK_SIZE],
            chorus_output_left: [0_f32; crate::BLOCK_SIZE],
            chorus_output_right: [0_f32; crate::BLOCK_SIZE],
        }
    }
}
