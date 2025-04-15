use std::error;
use std::fmt;
use std::io;

use crate::four_cc::FourCC;

/// Represents an error when loading a SoundFont.
#[derive(Debug)]
pub enum SoundFontError {
    IoError(io::Error),
    RiffChunkNotFound,
    InvalidRiffChunkType {
        expected: FourCC,
        actual: FourCC,
    },
    ListChunkNotFound,
    InvalidListChunkType {
        expected: FourCC,
        actual: FourCC,
    },
    ListContainsUnknownId(FourCC),
    SampleDataNotFound,
    UnsupportedSampleFormat,
    SubChunkNotFound(FourCC),
    InvalidPresetList,
    InvalidInstrumentId {
        preset_id: usize,
        instrument_id: usize,
    },
    InvalidPreset(usize),
    PresetNotFound,
    InvalidInstrumentList,
    InvalidSampleId {
        instrument_id: usize,
        sample_id: usize,
    },
    InvalidInstrument(usize),
    InstrumentNotFound,
    InvalidSampleHeaderList,
    InvalidZoneList,
    ZoneNotFound,
    InvalidGeneratorList,
    SanityCheckFailed,
}

impl error::Error for SoundFontError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SoundFontError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for SoundFontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SoundFontError::IoError(err) => err.fmt(f),
            SoundFontError::RiffChunkNotFound => write!(f, "the RIFF chunk was not found"),
            SoundFontError::InvalidRiffChunkType { expected, actual } => write!(
                f,
                "the type of the RIFF chunk must be '{}', but was '{}'",
                expected, actual
            ),
            SoundFontError::ListChunkNotFound => write!(f, "the LIST chunk was not found"),
            SoundFontError::InvalidListChunkType { expected, actual } => write!(
                f,
                "the type of the LIST chunk must be '{}', but was '{}'",
                expected, actual
            ),
            SoundFontError::ListContainsUnknownId(id) => {
                write!(f, "the INFO list contains an unknown ID '{id}'")
            }
            SoundFontError::SampleDataNotFound => write!(f, "no valid sample data was found"),
            SoundFontError::UnsupportedSampleFormat => write!(f, "SoundFont3 is not yet supported"),
            SoundFontError::SubChunkNotFound(id) => {
                write!(f, "the '{}' sub-chunk was not found", id)
            }
            SoundFontError::InvalidPresetList => write!(f, "the preset list is invalid"),
            SoundFontError::InvalidInstrumentId {
                preset_id,
                instrument_id,
            } => write!(
                f,
                "the preset with the ID '{preset_id}' contains an invalid instrument ID '{instrument_id}'"
            ),
            SoundFontError::InvalidPreset(preset_id) => {
                write!(f, "the preset with the ID '{preset_id}' has no zone")
            }
            SoundFontError::PresetNotFound => write!(f, "no valid preset was found"),
            SoundFontError::InvalidInstrumentList => write!(f, "the instrument list is invalid"),
            SoundFontError::InvalidSampleId {
                instrument_id,
                sample_id,
            } => write!(
                f,
                "the instrument with the ID '{instrument_id}' contains an invalid sample ID '{sample_id}'"
            ),
            SoundFontError::InvalidInstrument(instrument_id) => {
                write!(
                    f,
                    "the instrument with the ID '{instrument_id}' has no zone"
                )
            }
            SoundFontError::InstrumentNotFound => write!(f, "no valid instrument was found"),
            SoundFontError::InvalidSampleHeaderList => {
                write!(f, "the sample header list is invalid")
            }
            SoundFontError::InvalidZoneList => write!(f, "the zone list is invalid"),
            SoundFontError::ZoneNotFound => write!(f, "no valid zone was found"),
            SoundFontError::InvalidGeneratorList => write!(f, "the generator list is invalid"),
            SoundFontError::SanityCheckFailed => write!(f, "sanity check failed"),
        }
    }
}

impl From<io::Error> for SoundFontError {
    fn from(err: io::Error) -> Self {
        SoundFontError::IoError(err)
    }
}
