mod error;

mod binary_reader;
mod four_cc;
mod read_counter;

mod generator;
mod generator_type;
mod instrument;
mod instrument_info;
mod instrument_region;
mod preset;
mod preset_info;
mod preset_region;
mod region_pair;
mod sample_header;
mod soundfont;
mod soundfont_info;
mod soundfont_parameters;
mod soundfont_sampledata;
mod soundfont_version;
mod zone;
mod zone_info;

pub use self::soundfont::SoundFont;
use anyhow::{anyhow, Result};
use region_pair::RegionPair;
use rustysynth::SoundSource;
use std::collections::HashMap;

pub struct SoundFontProc {
    sound_font: SoundFont,
    preset_lookup: HashMap<i32, usize>,
    default_preset: usize,
}

impl SoundFontProc {
    pub fn new(sound_font: SoundFont) -> Self {
        let mut preset_lookup = HashMap::new();

        let mut min_preset_id = i32::MAX;
        let mut default_preset: usize = 0;
        for i in 0..sound_font.presets.len() {
            let preset = &sound_font.presets[i];

            // The preset ID is Int32, where the upper 16 bits represent the bank number
            // and the lower 16 bits represent the patch number.
            // This ID is used to search for presets by the combination of bank number
            // and patch number.
            let preset_id = (preset.bank_number << 16) | preset.patch_number;
            preset_lookup.insert(preset_id, i);

            // The preset with the minimum ID number will be default.
            // If the SoundFont is GM compatible, the piano will be chosen.
            if preset_id < min_preset_id {
                default_preset = i;
                min_preset_id = preset_id;
            }
        }

        Self {
            sound_font,
            preset_lookup,
            default_preset,
        }
    }
}

impl From<SoundFont> for SoundFontProc {
    fn from(sound_font: SoundFont) -> Self {
        Self::new(sound_font)
    }
}

impl SoundSource for SoundFontProc {
    fn wave_data(&self) -> &Vec<i16> {
        &self.sound_font.wave_data
    }
    #[allow(refining_impl_trait)]
    fn get_regions(
        &self,
        bank_id: i32,
        patch_id: i32,
        key: i32,
        velocity: i32,
    ) -> Result<RegionPair> {
        let preset_id = (bank_id << 16) | patch_id;
        let mut preset = self.default_preset;
        match self.preset_lookup.get(&preset_id) {
            Some(value) => preset = *value,
            None => {
                // Try fallback to the GM sound set.
                // Normally, the given patch number + the bank number 0 will work.
                // For drums (bank number >= 128), it seems to be better to select the standard set (128:0).
                let gm_preset_id = if bank_id < 128 { patch_id } else { 128 << 16 };

                // If no corresponding preset was found. Use the default one...
                if let Some(value) = self.preset_lookup.get(&gm_preset_id) {
                    preset = *value
                }
            }
        }

        let preset = &self.sound_font.presets[preset];
        for preset_region in preset.regions.iter() {
            if preset_region.contains(key, velocity) {
                let instrument = &self.sound_font.instruments[preset_region.instrument];
                for instrument_region in instrument.regions.iter() {
                    if instrument_region.contains(key, velocity) {
                        let region_pair = RegionPair::new(preset_region, instrument_region);
                        return Ok(region_pair);
                    }
                }
            }
        }
        Err(anyhow!(
            "No regions found for bank_id: {}, patch_id: {}, key: {}, velocity: {}",
            bank_id,
            patch_id,
            key,
            velocity
        ))
    }
}
