pub mod soundfont_math;

mod bi_quad_filter;
mod channel;
mod lfo;
mod modulation_envelope;
mod oscillator;
mod synthesizer;
mod voice;
mod volume_envelope;

// XXX chorus
// XXX echo
// XXX delay
mod reverb;

pub use self::oscillator::View;
pub use self::synthesizer::{Sound, SoundSource, Synthesizer};

pub const SAMPLE_RATE: i32 = 44100;

/// Specifies how the sample loops during playback.
#[derive(Debug, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// The sample will be played without loop.
    #[default]
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
