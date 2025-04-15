#[derive(Debug, PartialEq, Eq, Default)]
enum DataType {
    #[default]
    None,
    Rpn,
    Nrpn,
}

#[derive(Debug, Default)]
pub(crate) struct Channel {
    // XXX switch to u16
    preset_id: i16,

    modulation: i16,
    volume: i16,
    pan: i16,
    expression: i16,
    hold_pedal: bool,

    reverb_send: u8,
    chorus_send: u8,

    rpn: i16,
    pitch_bend_range: i16,
    coarse_tune: i16,
    fine_tune: i16,

    pitch_bend: f32,

    last_data_type: DataType,
}

macro_rules! set_coarse_fine {
    ($field:ident, $coarse:ident, $fine:ident) => {
        pub(crate) fn $coarse(&mut self, value: u8) {
            //let value = (value & 0x7F) as u16; // XXX
            let value = value as i16;
            self.$field = (self.$field & 0x7F) | (value << 7) as i16;
        }

        pub(crate) fn $fine(&mut self, value: u8) {
            //let value = (value & 0x7F) as u16; // XXX
            let value = value as i32;
            self.$field = (((self.$field as i32) & 0xFF80) | value) as i16
        }
    };
}

impl Channel {
    pub(crate) fn reset(&mut self) {
        self.preset_id = 0;
        self.volume = 100 << 7;
        self.pan = 64 << 7;
        self.reverb_send = 40;
        self.chorus_send = 0;
        self.pitch_bend_range = 2 << 7;
        self.coarse_tune = 0;
        self.fine_tune = 8192;
        self.reset_all_controllers();
    }

    pub(crate) fn reset_all_controllers(&mut self) {
        self.modulation = 0;
        self.expression = 127 << 7;
        self.hold_pedal = false;
        self.rpn = -1;
        self.pitch_bend = 0.0;
    }

    set_coarse_fine!(preset_id, set_bank, set_patch);
    set_coarse_fine!(modulation, set_modulation_coarse, set_modulation_fine);
    set_coarse_fine!(volume, set_volume_coarse, set_volume_fine);
    set_coarse_fine!(pan, set_pan_coarse, set_pan_fine);
    set_coarse_fine!(expression, set_expression_coarse, set_expression_fine);

    pub(crate) fn set_hold_pedal(&mut self, value: u8) {
        self.hold_pedal = value >= 64;
    }

    pub(crate) fn set_reverb_send(&mut self, value: u8) {
        self.reverb_send = value;
    }

    pub(crate) fn set_chorus_send(&mut self, value: u8) {
        self.chorus_send = value;
    }

    set_coarse_fine!(rpn, set_rpn_coarse_, set_rpn_fine_);
    pub(crate) fn set_rpn_coarse(&mut self, value: u8) {
        self.set_rpn_coarse_(value);
        self.last_data_type = DataType::Rpn;
    }

    pub(crate) fn set_rpn_fine(&mut self, value: u8) {
        self.set_rpn_fine_(value);
        self.last_data_type = DataType::Rpn;
    }

    pub(crate) fn set_nrpn_coarse(&mut self, _value: u8) {
        self.last_data_type = DataType::Nrpn;
    }

    pub(crate) fn set_nrpn_fine(&mut self, _value: u8) {
        self.last_data_type = DataType::Nrpn;
    }

    set_coarse_fine!(pitch_bend_range, set_pbr_coarse, set_pbr_fine);
    set_coarse_fine!(fine_tune, set_fine_tune_coarse, set_fine_tune_fine);
    pub(crate) fn data_entry_coarse(&mut self, value: u8) {
        if self.last_data_type != DataType::Rpn {
            return;
        }

        if self.rpn == 0 {
            self.set_pbr_coarse(value);
        } else if self.rpn == 1 {
            self.set_fine_tune_coarse(value);
        } else if self.rpn == 2 {
            self.coarse_tune = value as i16 - 64;
        }
    }

    pub(crate) fn data_entry_fine(&mut self, value: u8) {
        if self.last_data_type != DataType::Rpn {
            return;
        }

        if self.rpn == 0 {
            self.set_pbr_fine(value);
        } else if self.rpn == 1 {
            self.set_fine_tune_fine(value);
        }
    }

    pub(crate) fn set_pitch_bend(&mut self, value: u16) {
        let value = value as i32;
        self.pitch_bend = (1.0 / 8192.0) * (value - 8192) as f32;
    }

    pub(crate) fn get_preset_id(&self) -> u16 {
        self.preset_id as u16
    }

    pub(crate) fn get_modulation(&self) -> f32 {
        (50.0 / 16383.0) * self.modulation as f32
    }

    pub(crate) fn get_volume(&self) -> f32 {
        (1.0 / 16383.0) * self.volume as f32
    }

    pub(crate) fn get_pan(&self) -> f32 {
        (100.0 / 16383.0) * self.pan as f32 - 50.0
    }

    pub(crate) fn get_expression(&self) -> f32 {
        (1.0 / 16383.0) * self.expression as f32
    }

    pub(crate) fn get_hold_pedal(&self) -> bool {
        self.hold_pedal
    }

    pub(crate) fn get_reverb_send(&self) -> f32 {
        (1.0 / 127.0) * self.reverb_send as f32
    }

    pub(crate) fn get_chorus_send(&self) -> f32 {
        (1.0 / 127.0) * self.chorus_send as f32
    }

    pub(crate) fn get_pitch_bend_range(&self) -> f32 {
        (self.pitch_bend_range >> 7) as f32 + 0.01 * (self.pitch_bend_range & 0x7F) as f32
    }

    pub(crate) fn get_tune(&self) -> f32 {
        self.coarse_tune as f32 + (1.0 / 8192.0) * (self.fine_tune - 8192) as f32
    }

    pub(crate) fn get_pitch_bend(&self) -> f32 {
        self.get_pitch_bend_range() * self.pitch_bend
    }
}
