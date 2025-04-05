#[derive(Debug, Default)]
pub(crate) struct Lfo {
    active: bool,

    delay: f64,
    period: f64,

    current_time: f64,
    value: f32,
}

impl Lfo {
    pub(crate) fn start(&mut self, delay: f32, frequency: f32) {
        if frequency > 1.0E-3 {
            self.active = true;

            self.delay = delay as f64;
            self.period = 1.0 / frequency as f64;

            self.current_time = 0.0;
            self.value = 0.0;
        } else {
            self.active = false;
            self.value = 0.0;
        }
    }

    pub(crate) fn render(&mut self) -> f32 {
        if !self.active {
            return self.value;
        }

        self.current_time += 1.0;

        self.value = if self.current_time < self.delay {
            0_f32
        } else {
            let phase = ((self.current_time - self.delay) % self.period) / self.period;
            if phase < 0.25 {
                (4_f64 * phase) as f32
            } else if phase < 0.75 {
                (4_f64 * (0.5 - phase)) as f32
            } else {
                (4_f64 * (phase - 1.0)) as f32
            }
        };
        self.value
    }
}
