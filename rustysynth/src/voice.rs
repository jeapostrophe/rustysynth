use crate::bi_quad_filter::BiQuadFilter;
use crate::channel::Channel;
use crate::lfo::Lfo;
use crate::modulation_envelope::ModulationEnvelope;
use crate::oscillator::Oscillator;
use crate::soundfont_math::*;
use crate::synthesizer::Sound;
use crate::volume_envelope::VolumeEnvelope;
use std::f32::consts;

#[derive(Debug, Default, Eq, PartialEq)]
enum VoiceState {
    #[default]
    Playing,
    ReleaseRequested,
    Released,
}

#[derive(Debug, Default)]
pub(crate) struct Voice {
    vol_env: VolumeEnvelope,
    mod_env: ModulationEnvelope,

    vib_lfo: Lfo,
    mod_lfo: Lfo,

    oscillator: Oscillator,
    filter: BiQuadFilter,

    // A sudden change in the mix gain will cause pop noise.
    // To avoid this, we save the mix gain of the previous block,
    // and smooth out the gain if the gap between the current and previous gain is too large.
    // The actual smoothing process is done in the WriteBlock method of the Synthesizer class.
    pub(crate) previous_mix_gain_left: f32,
    pub(crate) previous_mix_gain_right: f32,
    pub(crate) current_mix_gain_left: f32,
    pub(crate) current_mix_gain_right: f32,

    pub(crate) previous_reverb_send: f32,
    pub(crate) previous_chorus_send: f32,
    pub(crate) current_reverb_send: f32,
    pub(crate) current_chorus_send: f32,

    pub(crate) channel: i32,
    pub(crate) key: i32,
    pub(crate) velocity: i32,

    note_gain: f32,

    cutoff: f32,
    resonance: f32,

    vib_lfo_to_pitch: f32,
    mod_lfo_to_pitch: f32,
    mod_env_to_pitch: f32,

    mod_lfo_to_cutoff: i32,
    mod_env_to_cutoff: i32,
    dynamic_cutoff: bool,

    mod_lfo_to_volume: f32,
    dynamic_volume: bool,

    instrument_pan: f32,
    instrument_reverb: f32,
    instrument_chorus: f32,

    // Some instruments require fast cutoff change, which can cause pop noise.
    // This is used to smooth out the cutoff frequency.
    smoothed_cutoff: f32,

    voice_state: VoiceState,
    pub(crate) voice_length: usize,
}

impl Voice {
    pub(crate) fn start<S: Sound>(&mut self, region: &S, channel: i32, key: i32, velocity: i32) {
        self.channel = channel;
        self.key = key;
        self.velocity = velocity;

        if velocity > 0 {
            // According to the Polyphone's implementation, the initial attenuation should be reduced to 40%.
            // I'm not sure why, but this indeed improves the loudness variability.
            let sample_attenuation = 0.4 * region.get_initial_attenuation();
            let decibels = 2.0 * linear_to_decibels(velocity as f32 / 127.0) - sample_attenuation;
            self.note_gain = decibels_to_linear(decibels);
        } else {
            self.note_gain = 0.0;
        }

        self.cutoff = region.get_initial_filter_cutoff_frequency();
        // XXX remove constant fields
        self.resonance = 1.0;

        self.vib_lfo_to_pitch = 0.0;
        self.mod_lfo_to_pitch = 0.0;
        self.mod_env_to_pitch = 0.0;

        self.mod_lfo_to_cutoff = 0;
        self.mod_env_to_cutoff = 0;
        self.dynamic_cutoff = false;

        self.mod_lfo_to_volume = 0.0;
        self.dynamic_volume = self.mod_lfo_to_volume > 0.05;

        self.instrument_pan = 0.0;
        self.instrument_reverb = 0.01 * region.get_reverb_effects_send();
        self.instrument_chorus = 0.0;

        self.vol_env.start(
            region.get_delay_volume_envelope(),
            region.get_attack_volume_envelope(),
            region.get_hold_volume_envelope(),
            region.get_decay_volume_envelope(),
            decibels_to_linear(-region.get_sustain_volume_envelope()),
            // If the release time is shorter than 10 ms, it will be clamped to 10 ms to avoid pop noise.
            region.get_release_volume_envelope().max(0.01),
        );
        self.mod_env.start(
            region.get_delay_modulation_envelope(),
            // According to the implementation of TinySoundFont, the attack time should be adjusted by the velocity.
            region.get_attack_modulation_envelope() * ((145 - velocity) as f32 / 144.0),
            region.get_hold_modulation_envelope(),
            region.get_decay_modulation_envelope(),
            region.get_release_modulation_envelope(),
        );
        self.vib_lfo.start(
            region.get_delay_vibrato_lfo(),
            region.get_frequency_vibrato_lfo(),
        );
        self.mod_lfo.start(
            region.get_delay_modulation_lfo(),
            region.get_frequency_modulation_lfo(),
        );
        self.oscillator.start(
            region.get_wave_data(),
            region.get_sample_modes(),
            region.sample_sample_rate(),
            region.get_sample_start_loop(),
            region.get_sample_end_loop(),
            region.get_root_key(),
            region.get_fine_tune(),
        );
        self.filter.clear_buffer();
        self.filter.set_low_pass_filter(self.cutoff, self.resonance);

        self.smoothed_cutoff = self.cutoff;

        self.voice_state = VoiceState::Playing;
        self.voice_length = 0;
    }

    pub(crate) fn end(&mut self) {
        if self.voice_state == VoiceState::Playing {
            self.voice_state = VoiceState::ReleaseRequested;
        }
    }

    pub(crate) fn kill(&mut self) {
        self.note_gain = 0.0;
    }

    pub(crate) fn render(&mut self, channel_info: &Channel) -> Option<f32> {
        if self.note_gain < NON_AUDIBLE {
            return None;
        }

        self.release_if_necessary(channel_info);

        let (vol_env_output, vol_env_on) = self.vol_env.render();
        if !vol_env_on {
            return None;
        }

        let mod_env_output = self.mod_env.render();
        let vib_lfo_output = self.vib_lfo.render();
        let mod_lfo_output = self.mod_lfo.render();

        let vib_pitch_change =
            (0.01 * channel_info.get_modulation() + self.vib_lfo_to_pitch) * vib_lfo_output;
        let mod_pitch_change =
            self.mod_lfo_to_pitch * mod_lfo_output + self.mod_env_to_pitch * mod_env_output;
        let channel_pitch_change = channel_info.get_tune() + channel_info.get_pitch_bend();
        let pitch = self.key as f32 + vib_pitch_change + mod_pitch_change + channel_pitch_change;

        let osc_output = self.oscillator.render(pitch);
        if osc_output.is_none() {
            return None;
        }
        let osc_output = osc_output.unwrap();

        if self.dynamic_cutoff {
            let cents = self.mod_lfo_to_cutoff as f32 * mod_lfo_output
                + self.mod_env_to_cutoff as f32 * mod_env_output;
            let factor = cents_to_multiplying_factor(cents);
            let new_cutoff = factor * self.cutoff;

            // The cutoff change is limited within x0.5 and x2 to reduce pop noise.
            let lower_limit = 0.5 * self.smoothed_cutoff;
            let upper_limit = 2.0 * self.smoothed_cutoff;
            self.smoothed_cutoff = new_cutoff.clamp(lower_limit, upper_limit);

            self.filter
                .set_low_pass_filter(self.smoothed_cutoff, self.resonance);
        }
        let output = self.filter.render(osc_output);

        self.previous_mix_gain_left = self.current_mix_gain_left;
        self.previous_mix_gain_right = self.current_mix_gain_right;
        self.previous_reverb_send = self.current_reverb_send;
        self.previous_chorus_send = self.current_chorus_send;

        // According to the GM spec, the following value should be squared.
        let ve = channel_info.get_volume() * channel_info.get_expression();
        let channel_gain = ve * ve;

        let mut mix_gain = self.note_gain * channel_gain * vol_env_output;
        if self.dynamic_volume {
            let decibels = self.mod_lfo_to_volume * mod_lfo_output;
            mix_gain *= decibels_to_linear(decibels);
        }

        let angle = (consts::PI / 200.0) * (channel_info.get_pan() + self.instrument_pan + 50.0);
        if angle <= 0.0 {
            self.current_mix_gain_left = mix_gain;
            self.current_mix_gain_right = 0.0;
        } else if angle >= HALF_PI {
            self.current_mix_gain_left = 0.0;
            self.current_mix_gain_right = mix_gain;
        } else {
            self.current_mix_gain_left = mix_gain * angle.cos();
            self.current_mix_gain_right = mix_gain * angle.sin();
        }

        self.current_reverb_send =
            (channel_info.get_reverb_send() + self.instrument_reverb).clamp(0.0, 1.0);
        self.current_chorus_send =
            (channel_info.get_chorus_send() + self.instrument_chorus).clamp(0.0, 1.0);

        if self.voice_length == 0 {
            self.previous_mix_gain_left = self.current_mix_gain_left;
            self.previous_mix_gain_right = self.current_mix_gain_right;
            self.previous_reverb_send = self.current_reverb_send;
            self.previous_chorus_send = self.current_chorus_send;
        }

        self.voice_length += 1;

        Some(output)
    }

    fn release_if_necessary(&mut self, channel_info: &Channel) {
        const MIN_VOICE_LENGTH: usize = (crate::SAMPLE_RATE / 500) as usize;
        if self.voice_length < MIN_VOICE_LENGTH {
            return;
        }

        if self.voice_state == VoiceState::ReleaseRequested && !channel_info.get_hold_pedal() {
            self.vol_env.release();
            self.mod_env.release();
            self.oscillator.release();

            self.voice_state = VoiceState::Released;
        }
    }

    pub(crate) fn get_priority(&self) -> f32 {
        if self.note_gain < NON_AUDIBLE {
            0.0
        } else {
            1.0 + self.vol_env.get_priority()
        }
    }
}
