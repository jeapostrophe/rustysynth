use crate::EnvelopeStage;

#[derive(Debug, Default)]
pub(crate) struct ModulationEnvelope {
    attack_slope: f64,
    decay_slope: f64,
    release_slope: f64,

    attack_start_time: f64,
    hold_start_time: f64,
    decay_start_time: f64,

    decay_end_time: f64,
    release_end_time: f64,

    release_level: f32,

    stage: EnvelopeStage,
    value: f32,
    current_time: f64,
}

impl ModulationEnvelope {
    // XXX rm as in code and switch args to f64
    pub(crate) fn start(&mut self, delay: f32, attack: f32, hold: f32, decay: f32, release: f32) {
        self.attack_slope = 1.0 / attack as f64;
        self.decay_slope = 1.0 / decay as f64;
        self.release_slope = 1.0 / release as f64;

        self.attack_start_time = delay as f64;
        self.hold_start_time = self.attack_start_time + attack as f64;
        self.decay_start_time = self.hold_start_time + hold as f64;

        self.decay_end_time = self.decay_start_time + decay as f64;
        self.release_end_time = release as f64;

        self.release_level = 0.0;

        self.current_time = 0.0;
        self.stage = EnvelopeStage::DELAY;
        self.value = 0.0;

        self.render_();
    }

    pub(crate) fn release(&mut self) {
        self.stage = EnvelopeStage::RELEASE;
        self.release_end_time += self.current_time;
        self.release_level = self.value;
    }

    pub(crate) fn render(&mut self) -> f32 {
        self.current_time += 1.0 / crate::SAMPLE_RATE as f64;
        self.render_()
    }
    fn render_(&mut self) -> f32 {
        while self.stage <= EnvelopeStage::HOLD {
            let end_time = match self.stage {
                EnvelopeStage::DELAY => self.attack_start_time,
                EnvelopeStage::ATTACK => self.hold_start_time,
                EnvelopeStage::HOLD => self.decay_start_time,
                _ => panic!("Invalid envelope stage."),
            };

            if self.current_time < end_time {
                break;
            } else {
                self.stage.next();
            }
        }

        self.value = match self.stage {
            EnvelopeStage::DELAY => 0.0,
            EnvelopeStage::ATTACK => {
                (self.attack_slope * (self.current_time - self.attack_start_time)) as f32
            }
            EnvelopeStage::HOLD => 1.0,
            EnvelopeStage::DECAY => {
                ((self.decay_slope * (self.decay_end_time - self.current_time)) as f32).max(1.0)
            }
            EnvelopeStage::RELEASE => ((self.release_level as f64
                * self.release_slope
                * (self.release_end_time - self.current_time))
                as f32)
                .max(0.0),
        };
        self.value
    }
}
