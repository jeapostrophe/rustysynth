use crate::channel::Channel;
use crate::chorus::Chorus;
use crate::reverb::Reverb;
use crate::soundfont_math::NON_AUDIBLE;
use crate::voice::Voice;
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
pub struct Synthesizer<Source, const CHANNELS: usize = 8, const VOICES: usize = 16> {
    pub(crate) sound_font: Source,
    channels: [Channel; CHANNELS],
    voices: [Voice; VOICES],
    master_volume: f32,
    reverb: Reverb,
    chorus: Chorus,
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

impl<Source: SoundSource, const CHANNELS: usize, const VOICES: usize>
    Synthesizer<Source, CHANNELS, VOICES>
{
    pub fn new<S>(sound_font_pre: S) -> Self
    where
        Source: From<S>,
    {
        Self {
            sound_font: sound_font_pre.into(),
            channels: core::array::from_fn(|_| Channel::default()),
            voices: core::array::from_fn(|_| Voice::default()),
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
        for voice in &mut self.voices {
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

        let voice_idx = self.allocate_voice();
        let channel_info = &self.channels[channel as usize];

        if let Ok(region_pair) = self.sound_font.get_regions(
            channel_info.get_bank_number(),
            channel_info.get_patch_number(),
            key,
            velocity,
        ) {
            self.voices[voice_idx].start(&region_pair, channel, key, velocity)
        }
    }

    fn allocate_voice(&mut self) -> usize {
        let voices = &mut self.voices;
        let mut candidate: usize = 0;

        // XXX use a simpler way to do this like .iter().min_by()
        // Too many active voices...
        // Find one which has the lowest priority.
        let mut lowest_priority = f32::MAX;
        for i in 0..voices.len() {
            let voice = &voices[i];
            let priority = voice.get_priority();
            if priority < lowest_priority {
                lowest_priority = priority;
                candidate = i;
            } else if priority == lowest_priority {
                // Same priority...
                // The older one should be more suitable for reuse.
                if voice.voice_length > voices[candidate].voice_length {
                    candidate = i;
                }
            }
        }

        candidate
    }

    fn note_off_all_(&mut self, channel: Option<i32>, immediate: bool) {
        for voice in &mut self.voices {
            let select = match channel {
                Some(ch) => voice.channel == ch,
                None => true,
            };
            if select {
                if immediate {
                    voice.kill();
                } else {
                    voice.end();
                }
            }
        }
    }

    pub fn note_off_all(&mut self, immediate: bool) {
        self.note_off_all_(None, immediate);
    }

    pub fn note_off_all_channel(&mut self, channel: i32, immediate: bool) {
        self.note_off_all_(Some(channel), immediate);
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
        self.note_off_all_(None, true);
        for channel in &mut self.channels {
            channel.reset();
        }
        self.reverb.mute();
        self.chorus.mute();
    }

    pub fn render(&mut self) -> (f32, f32) {
        fn multiply_add1(a: f32, x: f32, destination: &mut f32) {
            *destination += a * x;
        }

        fn write(previous_gain: f32, current_gain: f32, source: f32, destination: &mut f32) {
            if previous_gain.max(current_gain) < NON_AUDIBLE {
                return;
            }

            if (current_gain - previous_gain).abs() < 1.0E-3_f32 {
                multiply_add1(current_gain, source, destination);
            } else {
                multiply_add1(previous_gain, source, destination);
            }
        }

        let mut left = 0.0;
        let mut right = 0.0;
        // XXX Add back in reverb and chorus
        let mut reverb_input = 0.0;
        let mut chorus_input_left = 0.0;
        let mut chorus_input_right = 0.0;

        let data = self.sound_font.wave_data();

        for voice in &mut self.voices {
            let channel_info = &self.channels[voice.channel as usize];
            let vo = voice.render(data, channel_info);
            if vo.is_none() {
                continue;
            }
            let voice_out = vo.unwrap();
            // Normal output
            write(
                self.master_volume * voice.previous_mix_gain_left,
                self.master_volume * voice.current_mix_gain_left,
                voice_out,
                &mut left,
            );
            write(
                self.master_volume * voice.previous_mix_gain_right,
                self.master_volume * voice.current_mix_gain_right,
                voice_out,
                &mut right,
            );

            // Chorus
            /*
            write(
                voice.previous_chorus_send * voice.previous_mix_gain_left,
                voice.current_chorus_send * voice.current_mix_gain_left,
                *voice_out,
                &mut chorus_input_left,
            );
            write(
                voice.previous_chorus_send * voice.previous_mix_gain_right,
                voice.current_chorus_send * voice.current_mix_gain_right,
                *voice_out,
                &mut chorus_input_right,
            );
            */

            // Reverb
            /*
            write(
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
                multiply_add1(self.master_volume, chorus_output_left, &mut left);
                multiply_add1(self.master_volume, chorus_output_right, &mut right);

                let (reverb_output_left, reverb_output_right) = self.reverb.render(reverb_input);
                multiply_add1(self.master_volume, reverb_output_left, &mut left);
                multiply_add1(self.master_volume, reverb_output_right, &mut right);
        */

        (left, right)
    }
}
