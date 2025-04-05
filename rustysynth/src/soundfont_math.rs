use std::f32::consts;

pub const HALF_PI: f32 = consts::PI / 2.0;
pub const NON_AUDIBLE: f32 = 1.0e-3;
pub const LOG_NON_AUDIBLE: f32 = -6.907_755_4;

pub fn timecents_to_seconds(x: f32) -> f32 {
    2_f32.powf((1.0 / 1200.0) * x)
}

pub fn cents_to_hertz(x: f32) -> f32 {
    8.176 * 2_f32.powf((1.0 / 1200.0) * x)
}

pub fn cents_to_multiplying_factor(x: f32) -> f32 {
    2_f32.powf((1.0 / 1200.0) * x)
}

pub fn decibels_to_linear(x: f32) -> f32 {
    10_f32.powf(0.05 * x)
}

pub fn linear_to_decibels(x: f32) -> f32 {
    20.0 * x.log10()
}

pub fn key_number_to_multiplying_factor(cents: i32, key: i32) -> f32 {
    timecents_to_seconds((cents * (60 - key)) as f32)
}

pub fn exp_cutoff(x: f64) -> f64 {
    if x < LOG_NON_AUDIBLE as f64 {
        0_f64
    } else {
        x.exp()
    }
}
