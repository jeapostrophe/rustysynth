// From https://github.com/MindBuffer/lanceverb/tree/master

/// Delay-line whose maximum size is fixed
/// The advantage of using a static versus dynamic array is that its elements
/// can be laid out in a predicatable location in memeory. This can improve
/// access speeds if many delay-lines are used within another object, like a
/// reverb
#[derive(Copy, Clone, Debug)]
pub struct DelayLine<const N: usize> {
    pos: usize,
    buffer: [f32; N],
}

impl<const N: usize> Default for DelayLine<N> {
    /// Default constructor for a delay line
    fn default() -> Self {
        DelayLine {
            pos: 0,
            buffer: core::array::from_fn(|_| 0.0),
        }
    }
}

impl<const N: usize> DelayLine<N> {
    /// Get element at back
    pub fn back(&self) -> f32 {
        let idx = self.index_back();
        self.buffer[idx]
    }

    /// Get index of back element.
    pub fn index_back(&self) -> usize {
        let i = self.pos + 1;
        if i < N {
            i
        } else {
            0
        }
    }

    /// Read value at delay i
    pub fn read(&self, i: i32) -> &f32 {
        let mut idx = self.pos as i32 - i;
        if idx < 0 {
            idx += N as i32;
        }
        &self.buffer[idx as usize]
    }

    /// Write value to delay
    pub fn write(&mut self, value: f32) {
        self.buffer[self.pos] = value;
        self.pos += 1;
        if self.pos >= N {
            self.pos = 0;
        }
    }

    /// Write new value and return oldest value
    pub fn get_write_and_step(&mut self, value: f32) -> f32 {
        let r = self.buffer[self.pos];
        self.write(value);
        r
    }

    /// Comb filter input using a delay time equal to the maximum size of the delay-line
    pub fn comb(&mut self, value: f32, feed_fwd: f32, feed_bck: f32) -> f32 {
        let d = self.buffer[self.pos];
        let r = value + d * feed_bck;
        self.write(r);
        d + r * feed_fwd
    }

    /// Allpass filter input using a delay time equal to the maximum size of the delay-line
    pub fn allpass(&mut self, value: f32, feed_fwd: f32) -> f32 {
        self.comb(value, feed_fwd, -feed_fwd)
    }
}

#[derive(Copy, Clone, Debug)]
struct OnePole {
    one: f32,
    a: f32,
    b: f32,
}

impl Default for OnePole {
    fn default() -> OnePole {
        OnePole {
            one: 0.0,
            a: 1.0,
            b: 0.0,
        }
    }
}
impl OnePole {
    pub fn damping(&mut self, value: f32) {
        self.a = 1.0 - value.abs();
        self.b = value;
    }

    pub fn call(&mut self, i: f32) -> f32 {
        self.one = i * self.a + self.one * self.b;
        self.one
    }
}

/// Plate Reverberator
///
/// Design from:
///
/// Dattorro, J (1997). Effect design: Part 1: Reverberator and other filters.
///
/// Journal of Audio Engineering Society
/// [45(9):660-684](https://ccrma.stanford.edu/~dattorro/EffectDesignPart1.pdf)
#[derive(Clone, Debug, Default)]
pub struct Reverb {
    delay_feed_1: f32,
    delay_feed_2: f32,
    decay_1: f32,
    decay_2: f32,
    decay: f32,

    pre_delay: DelayLine<10>,
    one_pole: OnePole,
    all_pass_in_1: DelayLine<142>,
    all_pass_in_2: DelayLine<107>,
    all_pass_in_3: DelayLine<379>,
    all_pass_in_4: DelayLine<277>,

    all_pass_decay_11: DelayLine<672>,
    all_pass_decay_12: DelayLine<1800>,

    delay_11: DelayLine<4453>,
    delay_12: DelayLine<3720>,

    one_pole_1: OnePole,
    all_pass_decay_21: DelayLine<908>,
    all_pass_decay_22: DelayLine<2656>,

    delay_21: DelayLine<4217>,
    delay_22: DelayLine<3163>,

    one_pole_2: OnePole,
}

impl Reverb {
    /// Contructor default reverb
    pub fn reset(&mut self) -> () {
        *self = Reverb::default();
        self.bandwidth(0.9995);
        self.decay(0.85);
        self.damping(0.9);
        self.diffusion(0.76, 0.666, 0.707, 0.517);
    }

    /// Set input signal bandwidth, in [0,1]
    /// This sets the cutoff frequency of a one-pole low-pass filter on the
    /// input signal.
    pub fn bandwidth(&mut self, value: f32) -> &mut Reverb {
        self.one_pole_1.damping(1.0 - value);
        self
    }

    /// Set high-frequency damping amount, in [0,1]
    /// Higher amounts will dampen the diffuse sound more quickly.
    /// rather than high frequencies.
    pub fn damping(&mut self, value: f32) -> &mut Reverb {
        self.one_pole_1.damping(value);
        self.one_pole_2.damping(value);
        self
    }

    /// Set decay factor, in [0,1]
    pub fn decay(&mut self, value: f32) -> &mut Reverb {
        self.decay = value;
        self
    }

    /// Set diffusion amounts, in [0, 1]
    /// Values near 0.7 are recommended. Moving further away from 0.7 will lead
    /// to more distinct echoes.
    pub fn diffusion(&mut self, in_1: f32, in_2: f32, decay_1: f32, decay_2: f32) -> &mut Reverb {
        self.delay_feed_1 = in_1;
        self.delay_feed_2 = in_2;
        self.decay_1 = decay_1;
        self.decay_2 = decay_2;
        self
    }

    /// Set input diffusion 1 amount, [0,1]
    pub fn diffusion1(&mut self, value: f32) -> &mut Reverb {
        self.delay_feed_1 = value;
        self
    }

    /// Set input diffusion 2 amount, [0,1]
    pub fn diffusion2(&mut self, value: f32) -> &mut Reverb {
        self.delay_feed_2 = value;
        self
    }

    /// Set tank decay diffusion 1 amount, [0,1]
    pub fn diffusion_decay_1(&mut self, value: f32) -> &mut Reverb {
        self.decay_1 = value;
        self
    }

    /// Set tank decay diffusion 2 amount, [0,1]
    pub fn diffusion_decay_2(&mut self, value: f32) -> &mut Reverb {
        self.decay_2 = value;
        self
    }

    /// Compute wet stereo output from dry mono input
    /// @param[ in] in      dry input sample
    /// @param[out] out1    wet output sample 1
    /// @param[out] out2    wet output sample 2    
    pub fn render(&mut self, input: f32) -> (f32, f32) {
        let mut value = self.pre_delay.get_write_and_step(input * 0.5);
        value = self.one_pole.call(value);
        value = self.all_pass_in_1.allpass(value, self.delay_feed_1);
        value = self.all_pass_in_2.allpass(value, self.delay_feed_1);
        value = self.all_pass_in_3.allpass(value, self.delay_feed_2);
        value = self.all_pass_in_4.allpass(value, self.delay_feed_2);

        let mut a = value + self.delay_22.back() * self.decay;
        let mut b = value + self.delay_12.back() * self.decay;

        a = self.all_pass_decay_11.allpass(a, -self.decay_1);
        a = self.delay_11.get_write_and_step(a);
        a = self.one_pole_1.call(a) * self.decay;
        a = self.all_pass_decay_12.allpass(a, self.decay_2);
        self.delay_12.write(a);

        b = self.all_pass_decay_21.allpass(b, -self.decay_1);
        b = self.delay_21.get_write_and_step(b);
        b = self.one_pole_2.call(b) * self.decay;
        b = self.all_pass_decay_22.allpass(b, self.decay_2);
        self.delay_22.write(b);

        let output_1 = {
            self.delay_21.read(266) + self.delay_21.read(2974) - self.all_pass_decay_22.read(1913)
                + self.delay_22.read(1996)
                - self.delay_11.read(1990)
                - self.all_pass_decay_12.read(187)
                - self.delay_12.read(1066)
        };

        let output_2 = {
            self.delay_11.read(353) + self.delay_11.read(3627) - self.all_pass_decay_12.read(1228)
                + self.delay_12.read(2673)
                - self.delay_21.read(2111)
                - self.all_pass_decay_22.read(335)
                - self.delay_22.read(121)
        };

        (output_1, output_2)
    }
}
