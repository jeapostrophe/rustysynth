use crate::lfo::Lfo;
use crate::modulation_envelope::ModulationEnvelope;
use crate::oscillator::Oscillator;
use crate::soundfont_math::*;
use crate::synthesizer::Sound;
use crate::volume_envelope::VolumeEnvelope;

pub(crate) fn start_oscillator<S: Sound>(oscillator: &mut Oscillator, region: &S) {
    let sample_rate = region.sample_sample_rate();
    let loop_mode = region.get_sample_modes();
    let start = region.get_sample_start();
    let end = region.get_sample_end();
    let start_loop = region.get_sample_start_loop();
    let end_loop = region.get_sample_end_loop();
    let root_key = region.get_root_key();
    let fine_tune = region.get_fine_tune();

    oscillator.start(
        loop_mode,
        sample_rate,
        start,
        end,
        start_loop,
        end_loop,
        root_key,
        fine_tune,
    );
}

pub(crate) fn start_volume_envelope<S: Sound>(envelope: &mut VolumeEnvelope, region: &S) {
    // If the release time is shorter than 10 ms, it will be clamped to 10 ms to avoid pop noise.

    let delay = region.get_delay_volume_envelope();
    let attack = region.get_attack_volume_envelope();
    let hold = region.get_hold_volume_envelope();
    let decay = region.get_decay_volume_envelope();
    let sustain = decibels_to_linear(-region.get_sustain_volume_envelope());
    let release = region.get_release_volume_envelope().max(0.01_f32);

    envelope.start(delay, attack, hold, decay, sustain, release);
}

pub(crate) fn start_modulation_envelope<S: Sound>(
    envelope: &mut ModulationEnvelope,
    region: &S,
    velocity: i32,
) {
    // According to the implementation of TinySoundFont, the attack time should be adjusted by the velocity.

    let delay = region.get_delay_modulation_envelope();
    let attack = region.get_attack_modulation_envelope() * ((145 - velocity) as f32 / 144_f32);
    let hold = region.get_hold_modulation_envelope();
    let decay = region.get_decay_modulation_envelope();
    let release = region.get_release_modulation_envelope();

    envelope.start(delay, attack, hold, decay, release);
}

pub(crate) fn start_vibrato<S: Sound>(lfo: &mut Lfo, region: &S, _key: i32, _velocity: i32) {
    lfo.start(
        region.get_delay_vibrato_lfo(),
        region.get_frequency_vibrato_lfo(),
    );
}

pub(crate) fn start_modulation<S: Sound>(lfo: &mut Lfo, region: &S, _key: i32, _velocity: i32) {
    lfo.start(
        region.get_delay_modulation_lfo(),
        region.get_frequency_modulation_lfo(),
    );
}
