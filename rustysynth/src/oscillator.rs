use crate::LoopMode;

// In this class, fixed-point numbers are used for speed-up.
// A fixed-point number is expressed by Int64, whose lower 24 bits represent the fraction part,
// and the rest represent the integer part.
// For clarity, fixed-point number variables have a suffix "_fp".

#[derive(Debug, Default)]
pub(crate) struct Oscillator {
    loop_mode: LoopMode,
    sample_sample_rate: i32,
    start: i32,
    end: i32,
    start_loop: i32,
    end_loop: i32,
    root_key: i32,

    tune: f32,
    sample_rate_ratio: f32,

    looping: bool,

    position_fp: i64,
}

impl Oscillator {
    const FRAC_BITS: i32 = 24;
    const FRAC_UNIT: i64 = 1_i64 << Oscillator::FRAC_BITS;
    const FP_TO_SAMPLE: f32 = 1.0 / (32768 * Oscillator::FRAC_UNIT) as f32;

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start(
        &mut self,
        loop_mode: LoopMode,
        sample_rate: i32,
        start: i32,
        end: i32,
        start_loop: i32,
        end_loop: i32,
        root_key: i32,
        fine_tune: i32,
    ) {
        self.loop_mode = loop_mode;
        self.sample_sample_rate = sample_rate;
        self.start = start;
        self.end = end;
        self.start_loop = start_loop;
        self.end_loop = end_loop;
        self.root_key = root_key;

        self.tune = 0.01 * fine_tune as f32;
        self.sample_rate_ratio = sample_rate as f32 / crate::SAMPLE_RATE as f32;
        self.looping = self.loop_mode != LoopMode::NoLoop;
        self.position_fp = (start as i64) << Oscillator::FRAC_BITS;
    }

    pub(crate) fn release(&mut self) {
        if self.loop_mode == LoopMode::LoopUntilNoteOff {
            self.looping = false;
        }
    }

    pub(crate) fn render(&mut self, data: &[i16], pitch: f32) -> Option<f32> {
        // XXX Improve this algorithm e.g. windowed sinc
        let (index1, index2) = if self.looping {
            let end_loop_fp = (self.end_loop as i64) << Oscillator::FRAC_BITS;
            let loop_length = (self.end_loop - self.start_loop) as i64;
            let loop_length_fp = loop_length << Oscillator::FRAC_BITS;

            if self.position_fp >= end_loop_fp {
                self.position_fp -= loop_length_fp;
            }

            let index1 = (self.position_fp >> Oscillator::FRAC_BITS) as usize;
            let mut index2 = index1 + 1;
            if index2 >= self.end_loop as usize {
                index2 -= loop_length as usize;
            }
            (index1, index2)
        } else {
            let index = (self.position_fp >> Oscillator::FRAC_BITS) as usize;
            if index >= self.end as usize {
                return None;
            }
            (index, index + 1)
        };

        let pitch_change = (pitch - self.root_key as f32) + self.tune;
        let pitch_ratio = (self.sample_rate_ratio * 2_f32.powf(pitch_change / 12.0)) as f64;
        let pitch_ratio_fp = (Oscillator::FRAC_UNIT as f64 * pitch_ratio) as i64;

        let x1 = data[index1] as i64;
        let x2 = data[index2] as i64;
        let a_fp = self.position_fp & (Oscillator::FRAC_UNIT - 1);
        self.position_fp += pitch_ratio_fp;
        Some(Oscillator::FP_TO_SAMPLE * ((x1 << Oscillator::FRAC_BITS) + a_fp * (x2 - x1)) as f32)
    }
}
