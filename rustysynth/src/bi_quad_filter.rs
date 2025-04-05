use std::f32::consts;

#[derive(Debug, Default)]
pub(crate) struct BiQuadFilter {
    active: bool,

    a0: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    a4: f32,

    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiQuadFilter {
    const RESONANCE_PEAK_OFFSET: f32 = 1.0 - 1.0 / core::f32::consts::SQRT_2;

    pub(crate) fn clear_buffer(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub(crate) fn set_low_pass_filter(&mut self, cutoff_frequency: f32, resonance: f32) {
        let sample_rate = crate::SAMPLE_RATE as f32;
        if cutoff_frequency < 0.499 * sample_rate {
            self.active = true;

            // This equation gives the Q value which makes the desired resonance peak.
            // The error of the resultant peak height is less than 3%.
            let q =
                resonance - BiQuadFilter::RESONANCE_PEAK_OFFSET / (1.0 + 6.0 * (resonance - 1.0));

            let w = 2.0 * consts::PI * cutoff_frequency / sample_rate;
            let cosw = w.cos();
            let alpha = w.sin() / (2.0 * q);

            let b0 = (1.0 - cosw) / 2.0;
            let b1 = 1.0 - cosw;
            let b2 = (1.0 - cosw) / 2.0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cosw;
            let a2 = 1.0 - alpha;

            self.set_coefficients(a0, a1, a2, b0, b1, b2);
        } else {
            self.active = false;
        }
    }

    pub(crate) fn render(&mut self, input: f32) -> f32 {
        let output = if self.active {
            self.a0 * input + self.a1 * self.x1 + self.a2 * self.x2
                - self.a3 * self.y1
                - self.a4 * self.y2
        } else {
            input
        };

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    fn set_coefficients(&mut self, a0: f32, a1: f32, a2: f32, b0: f32, b1: f32, b2: f32) {
        self.a0 = b0 / a0;
        self.a1 = b1 / a0;
        self.a2 = b2 / a0;
        self.a3 = a1 / a0;
        self.a4 = a2 / a0;
    }
}
