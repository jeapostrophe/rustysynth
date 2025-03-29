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
