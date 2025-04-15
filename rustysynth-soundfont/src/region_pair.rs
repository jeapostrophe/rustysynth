use crate::generator_type::GeneratorType;
use crate::instrument_region::InstrumentRegion;
use crate::preset_region::PresetRegion;
use rustysynth::soundfont_math::*;
use rustysynth::{LoopMode, Sound, View};

pub struct RegionPair<'a> {
    pub(crate) preset: &'a PresetRegion,
    pub(crate) instrument: &'a InstrumentRegion,
    pub(crate) wave_data: View<i16>,
}

impl Sound for RegionPair<'_> {
    fn get_wave_data(&self) -> View<i16> {
        self.wave_data.clone()
    }
    fn sample_sample_rate(&self) -> i32 {
        self.instrument.sample_sample_rate
    }
    fn get_sample_start_loop(&self) -> i32 {
        self.instrument.get_sample_start_loop() - self.instrument.sample_start
    }

    fn get_sample_end_loop(&self) -> i32 {
        self.instrument.get_sample_end_loop() - self.instrument.sample_start
    }

    fn get_initial_filter_cutoff_frequency(&self) -> f32 {
        cents_to_hertz(self.gs(GeneratorType::INITIAL_FILTER_CUTOFF_FREQUENCY as usize) as f32)
    }

    fn get_reverb_effects_send(&self) -> f32 {
        0.1_f32 * self.gs(GeneratorType::REVERB_EFFECTS_SEND as usize) as f32
    }

    fn get_delay_modulation_lfo(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::DELAY_MODULATION_LFO as usize) as f32)
    }

    fn get_frequency_modulation_lfo(&self) -> f32 {
        cents_to_hertz(self.gs(GeneratorType::FREQUENCY_MODULATION_LFO as usize) as f32)
    }

    fn get_delay_vibrato_lfo(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::DELAY_VIBRATO_LFO as usize) as f32)
    }

    fn get_frequency_vibrato_lfo(&self) -> f32 {
        cents_to_hertz(self.gs(GeneratorType::FREQUENCY_VIBRATO_LFO as usize) as f32)
    }

    fn get_delay_modulation_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::DELAY_MODULATION_ENVELOPE as usize) as f32)
    }

    fn get_attack_modulation_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::ATTACK_MODULATION_ENVELOPE as usize) as f32)
    }

    fn get_hold_modulation_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::HOLD_MODULATION_ENVELOPE as usize) as f32)
    }

    fn get_decay_modulation_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::DECAY_MODULATION_ENVELOPE as usize) as f32)
    }
    fn get_release_modulation_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::RELEASE_MODULATION_ENVELOPE as usize) as f32)
    }
    fn get_delay_volume_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::DELAY_VOLUME_ENVELOPE as usize) as f32)
    }

    fn get_attack_volume_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::ATTACK_VOLUME_ENVELOPE as usize) as f32)
    }

    fn get_hold_volume_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::HOLD_VOLUME_ENVELOPE as usize) as f32)
    }

    fn get_decay_volume_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::DECAY_VOLUME_ENVELOPE as usize) as f32)
    }

    fn get_sustain_volume_envelope(&self) -> f32 {
        0.1_f32 * self.gs(GeneratorType::SUSTAIN_VOLUME_ENVELOPE as usize) as f32
    }

    fn get_release_volume_envelope(&self) -> f32 {
        timecents_to_seconds(self.gs(GeneratorType::RELEASE_VOLUME_ENVELOPE as usize) as f32)
    }

    fn get_initial_attenuation(&self) -> f32 {
        0.1_f32 * self.gs(GeneratorType::INITIAL_ATTENUATION as usize) as f32
    }

    fn get_fine_tune(&self) -> i32 {
        self.gs(GeneratorType::FINE_TUNE as usize) + self.instrument.sample_pitch_correction
    }

    fn get_sample_modes(&self) -> LoopMode {
        self.instrument.get_sample_modes()
    }

    fn get_root_key(&self) -> i32 {
        self.instrument.get_root_key()
    }
}

impl RegionPair<'_> {
    fn gs(&self, i: usize) -> i32 {
        self.preset.gs[i] as i32 + self.instrument.gs[i] as i32
    }
}
