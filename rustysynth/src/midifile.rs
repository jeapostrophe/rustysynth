use anyhow::{anyhow, Result};
use std::{collections::VecDeque, io::Read};

/// Represents a standard MIDI file.
#[derive(Debug)]
pub struct MidiFile {
    pub(crate) events: Vec<MidiEvent>,
}
#[derive(Debug)]
pub struct MidiEvent {
    pub(crate) time: f64,
    pub(crate) ch: midly::num::u4,
    pub(crate) msg: midly::MidiMessage,
}

#[derive(Debug)]
pub struct TempoChange {
    time: f64,
    us_per_beat: f64,
}

impl MidiFile {
    pub fn new<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = vec![];
        reader.read_to_end(&mut buf)?;
        let smf = midly::Smf::parse(buf.as_slice())?;
        let ticks_per_beat = match smf.header.timing {
            midly::Timing::Metrical(tpb) => tpb.as_int() as f64,
            midly::Timing::Timecode(..) => return Err(anyhow!("Timecode is not supported")),
        };
        let mut tempo_changes: Vec<TempoChange> = vec![];
        let mut all_evts = vec![];
        for track in smf.tracks {
            let mut time = 0.0;
            let mut us_per_beat = 500_000.0;
            let mut tempo_idx = 0;
            let mut track_evts = VecDeque::new();
            for evt in track {
                let first_track = all_evts.is_empty();
                if !first_track {
                    while tempo_idx < tempo_changes.len() && tempo_changes[tempo_idx].time <= time {
                        us_per_beat = tempo_changes[tempo_idx].us_per_beat;
                        tempo_idx += 1;
                    }
                }
                let midly::TrackEvent { delta, kind } = evt;
                let delta_tick = delta.as_int() as f64;
                let delta_beats = delta_tick / ticks_per_beat; // T / (T/B) = T * (B/T) = B
                let delta_us = delta_beats * us_per_beat; // B * us/B = us
                let delta_s = delta_us / 1_000_000.0; // us / 1_000_000 = s
                time += delta_s;
                if first_track {
                    if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) = kind {
                        us_per_beat = tempo.as_int() as f64;
                        tempo_changes.push(TempoChange { time, us_per_beat });
                    }
                }
                match kind {
                    midly::TrackEventKind::Midi { channel, message } => {
                        track_evts.push_back(MidiEvent {
                            time,
                            ch: channel,
                            msg: message,
                        });
                    }
                    _ => {}
                }
            }
            all_evts.push(track_evts);
        }

        let mut events = vec![];
        while all_evts.iter().any(|evts| !evts.is_empty()) {
            let which = all_evts
                .iter()
                .enumerate()
                .map(|(i, evts)| {
                    (
                        i,
                        if let Some(evt) = evts.front() {
                            evt.time
                        } else {
                            f64::INFINITY
                        },
                    )
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            let which = &mut all_evts[which];
            let evt = which.pop_front().unwrap();
            events.push(evt);
        }

        Ok(Self { events })
    }

    /// Get the length of the MIDI file in seconds.
    pub fn get_length(&self) -> f64 {
        self.events.last().unwrap().time
    }
}
