use crate::soundfont_math::*;
use crate::EnvelopeStage;

#[derive(Debug, Default)]
pub(crate) struct VolumeEnvelope {
    attack_slope: f64,
    decay_slope: f64,
    release_slope: f64,

    attack_start_time: f64,
    hold_start_time: f64,
    decay_start_time: f64,
    release_start_time: f64,

    sustain_level: f32,
    release_level: f32,

    stage: EnvelopeStage,
    value: f32,
    current_time: f64,
}

impl VolumeEnvelope {
    pub(crate) fn start(
        &mut self,
        delay: f64,
        attack: f64,
        hold: f64,
        decay: f64,
        sustain: f32,
        release: f64,
    ) {
        self.attack_slope = 1.0 / attack;
        self.decay_slope = -9.226 / decay;
        self.release_slope = -9.226 / release;

        self.attack_start_time = delay;
        self.hold_start_time = self.attack_start_time + attack;
        self.decay_start_time = self.hold_start_time + hold;
        self.release_start_time = 0.0;

        self.sustain_level = sustain.clamp(0.0, 1.0);
        self.release_level = 0.0;

        self.stage = EnvelopeStage::DELAY;
        self.value = 0.0;
        self.current_time = 0.0;

        self.render_();
    }

    pub(crate) fn release(&mut self) {
        self.stage = EnvelopeStage::RELEASE;
        self.release_start_time = self.current_time;
        self.release_level = self.value;
    }

    pub(crate) fn render(&mut self) -> (f32, bool) {
        self.current_time += 1.0 / crate::SAMPLE_RATE as f64;
        self.render_()
    }
    fn render_(&mut self) -> (f32, bool) {
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

        let outb = match self.stage {
            EnvelopeStage::DELAY => {
                self.value = 0.0;
                true
            }
            EnvelopeStage::ATTACK => {
                self.value =
                    (self.attack_slope * (self.current_time - self.attack_start_time)) as f32;
                true
            }
            EnvelopeStage::HOLD => {
                self.value = 1.0;
                true
            }
            EnvelopeStage::DECAY => {
                self.value =
                    (exp_cutoff(self.decay_slope * (self.current_time - self.decay_start_time))
                        as f32)
                        .max(self.sustain_level);
                self.value > NON_AUDIBLE
            }
            EnvelopeStage::RELEASE => {
                self.value = (self.release_level as f64
                    * exp_cutoff(
                        self.release_slope * (self.current_time - self.release_start_time),
                    )) as f32;
                self.value > NON_AUDIBLE
            }
        };
        (self.value, outb)
    }

    pub(crate) fn get_priority(&self) -> u8 {
        self.stage.get_priority()
    }
}
