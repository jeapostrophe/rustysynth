pub mod soundfont_math;

mod bi_quad_filter;
mod channel;
mod lfo;
mod modulation_envelope;
mod oscillator;
mod region_ex;
mod synthesizer;
mod synthesizer_settings;
mod voice;
mod voice_collection;
mod volume_envelope;

mod chorus;
mod reverb;

pub use self::synthesizer::{Sound, SoundSource, Synthesizer};
pub use self::synthesizer_settings::SynthesizerSettings;

/// Specifies how the sample loops during playback.
#[derive(Debug, PartialEq, Eq)]
pub enum LoopMode {
    /// The sample will be played without loop.
    NoLoop,
    /// The sample will loop continuously.
    Continuous,
    /// The sample will loop until the note stops.
    LoopUntilNoteOff,
}

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum EnvelopeStage {
    #[default]
    DELAY,
    ATTACK,
    HOLD,
    DECAY,
    RELEASE,
}

impl EnvelopeStage {
    pub(crate) fn next(&mut self) {
        *self = match *self {
            EnvelopeStage::DELAY => EnvelopeStage::ATTACK,
            EnvelopeStage::ATTACK => EnvelopeStage::HOLD,
            EnvelopeStage::HOLD => EnvelopeStage::DECAY,
            EnvelopeStage::DECAY => EnvelopeStage::RELEASE,
            EnvelopeStage::RELEASE => EnvelopeStage::RELEASE,
        };
    }
}
