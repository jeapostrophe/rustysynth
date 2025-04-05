use crate::channel::Channel;
use crate::chorus::Chorus;
use crate::reverb::Reverb;
use crate::voice_collection::VoiceCollection;
use crate::LoopMode;
use anyhow::Result;

pub trait Sound {
    fn sample_sample_rate(&self) -> i32;
    fn get_sample_start(&self) -> i32;
    fn get_sample_end(&self) -> i32;
    fn get_sample_start_loop(&self) -> i32;
    fn get_sample_end_loop(&self) -> i32;
    fn get_initial_filter_cutoff_frequency(&self) -> f32;
    fn get_reverb_effects_send(&self) -> f32;
    fn get_delay_modulation_lfo(&self) -> f32;
    fn get_frequency_modulation_lfo(&self) -> f32;
    fn get_delay_vibrato_lfo(&self) -> f32;
    fn get_frequency_vibrato_lfo(&self) -> f32;
    fn get_delay_modulation_envelope(&self) -> f32;
    fn get_attack_modulation_envelope(&self) -> f32;
    fn get_hold_modulation_envelope(&self) -> f32;
    fn get_decay_modulation_envelope(&self) -> f32;
    fn get_release_modulation_envelope(&self) -> f32;
    fn get_delay_volume_envelope(&self) -> f32;
    fn get_attack_volume_envelope(&self) -> f32;
    fn get_hold_volume_envelope(&self) -> f32;
    fn get_decay_volume_envelope(&self) -> f32;
    fn get_sustain_volume_envelope(&self) -> f32;
    fn get_release_volume_envelope(&self) -> f32;
    fn get_initial_attenuation(&self) -> f32;
    fn get_fine_tune(&self) -> i32;
    fn get_sample_modes(&self) -> LoopMode;
    fn get_root_key(&self) -> i32;
    // XXX Add something to get a read-only Cursor on the sample data
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
    master_volume: f32,
    reverb: Reverb,
    chorus: Chorus,
}

pub const CHANNELS: usize = 16;

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

    pub fn multiply_add1(a: f32, x: f32, destination: &mut f32) {
        *destination += a * x;
    }

    pub fn write(previous_gain: f32, current_gain: f32, source: f32, destination: &mut f32) {
        if previous_gain.max(current_gain) < NON_AUDIBLE {
            return;
        }

        if (current_gain - previous_gain).abs() < 1.0E-3_f32 {
            multiply_add1(current_gain, source, destination);
        } else {
            multiply_add1(previous_gain, source, destination);
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
    pub fn new<S>(sound_font_pre: S) -> Self
    where
        Source: From<S>,
    {
        let mut channels: Vec<Channel> = Vec::new();
        for _ in 0..CHANNELS {
            channels.push(Channel::default());
        }

        Self {
            sound_font: sound_font_pre.into(),
            channels,
            voices: VoiceCollection::default(),
            master_volume: 0.5,
            reverb: Reverb::default(),
            chorus: Chorus::default(),
        }
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
        for voice in &mut self.voices.0 {
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
            for voice in &mut self.voices.0 {
                voice.end();
            }
        }
    }

    pub fn note_off_all_channel(&mut self, channel: i32, immediate: bool) {
        for voice in &mut self.voices.0 {
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
        self.master_volume = 0.5;
        self.voices.clear();
        for channel in &mut self.channels {
            channel.reset();
        }
        self.reverb.mute();
        self.chorus.mute();
    }

    pub fn render(&mut self) -> (f32, f32) {
        let voice_outs = self
            .voices
            .render(self.sound_font.wave_data(), &self.channels);

        let mut left = 0.0;
        let mut right = 0.0;
        // XXX Add back in reverb and chorus
        let mut reverb_input = 0.0;
        let mut chorus_input_left = 0.0;
        let mut chorus_input_right = 0.0;

        for (voice, voice_out) in self.voices.0.iter().zip(voice_outs.iter()) {
            // Normal output
            array_math::write(
                self.master_volume * voice.previous_mix_gain_left,
                self.master_volume * voice.current_mix_gain_left,
                *voice_out,
                &mut left,
            );
            array_math::write(
                self.master_volume * voice.previous_mix_gain_right,
                self.master_volume * voice.current_mix_gain_right,
                *voice_out,
                &mut right,
            );

            // Chorus
            /*
            array_math::write(
                voice.previous_chorus_send * voice.previous_mix_gain_left,
                voice.current_chorus_send * voice.current_mix_gain_left,
                *voice_out,
                &mut chorus_input_left,
            );
            array_math::write(
                voice.previous_chorus_send * voice.previous_mix_gain_right,
                voice.current_chorus_send * voice.current_mix_gain_right,
                *voice_out,
                &mut chorus_input_right,
            );
            */

            // Reverb
            /*
            array_math::write(
                self.reverb.get_input_gain()
                    * voice.previous_reverb_send
                    * (voice.previous_mix_gain_left + voice.previous_mix_gain_right),
                self.reverb.get_input_gain()
                    * voice.current_reverb_send
                    * (voice.current_mix_gain_left + voice.current_mix_gain_right),
                *voice_out,
                &mut reverb_input,
            );
            */
        }

        /* XXX
                let (chorus_output_left, chorus_output_right) =
                    self.chorus.render(chorus_input_left, chorus_input_right);
                array_math::multiply_add1(self.master_volume, chorus_output_left, &mut left);
                array_math::multiply_add1(self.master_volume, chorus_output_right, &mut right);

                let (reverb_output_left, reverb_output_right) = self.reverb.render(reverb_input);
                array_math::multiply_add1(self.master_volume, reverb_output_left, &mut left);
                array_math::multiply_add1(self.master_volume, reverb_output_right, &mut right);
        */

        (left, right)
    }
}
