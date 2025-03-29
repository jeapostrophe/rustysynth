use std::error;
use std::fmt;

/// Represents an error when initializing a synthesizer.
#[derive(Debug)]
pub enum SynthesizerError {
    SampleRateOutOfRange(i32),
    BlockSizeOutOfRange(usize),
    MaximumPolyphonyOutOfRange(usize),
}

impl error::Error for SynthesizerError {}

impl fmt::Display for SynthesizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SynthesizerError::SampleRateOutOfRange(value) => write!(
                f,
                "the sample rate must be between 16000 and 192000, but was {}",
                value
            ),
            SynthesizerError::BlockSizeOutOfRange(value) => write!(
                f,
                "the block size must be between 8 and 1024, but was {}",
                value
            ),
            SynthesizerError::MaximumPolyphonyOutOfRange(value) => {
                write!(
                    f,
                    "the maximum number of polyphony must be between 8 and 256, but was {}",
                    value
                )
            }
        }
    }
}

/// Specifies a set of parameters for synthesis.
#[derive(Debug)]
pub struct SynthesizerSettings {
    /// The sample rate for synthesis.
    pub sample_rate: i32,
    /// The block size for rendering waveform.
    pub block_size: usize,
    /// The number of maximum polyphony.
    pub maximum_polyphony: usize,
    /// The value indicating whether reverb and chorus are enabled.
    pub enable_reverb_and_chorus: bool,
}

impl SynthesizerSettings {
    const DEFAULT_BLOCK_SIZE: usize = 64;
    const DEFAULT_MAXIMUM_POLYPHONY: usize = 64;
    const DEFAULT_ENABLE_REVERB_AND_CHORUS: bool = true;

    /// Initializes a new instance of synthesizer settings.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate for synthesis.
    pub fn new(sample_rate: i32) -> Self {
        Self {
            sample_rate,
            block_size: SynthesizerSettings::DEFAULT_BLOCK_SIZE,
            maximum_polyphony: SynthesizerSettings::DEFAULT_MAXIMUM_POLYPHONY,
            enable_reverb_and_chorus: SynthesizerSettings::DEFAULT_ENABLE_REVERB_AND_CHORUS,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), SynthesizerError> {
        SynthesizerSettings::check_sample_rate(self.sample_rate)?;
        SynthesizerSettings::check_block_size(self.block_size)?;
        SynthesizerSettings::check_maximum_polyphony(self.maximum_polyphony)?;

        Ok(())
    }

    fn check_sample_rate(value: i32) -> Result<(), SynthesizerError> {
        if !(16_000..=192_000).contains(&value) {
            return Err(SynthesizerError::SampleRateOutOfRange(value));
        }

        Ok(())
    }

    fn check_block_size(value: usize) -> Result<(), SynthesizerError> {
        if !(8..=1024).contains(&value) {
            return Err(SynthesizerError::BlockSizeOutOfRange(value));
        }

        Ok(())
    }

    fn check_maximum_polyphony(value: usize) -> Result<(), SynthesizerError> {
        if !(8..=256).contains(&value) {
            return Err(SynthesizerError::MaximumPolyphonyOutOfRange(value));
        }

        Ok(())
    }
}
