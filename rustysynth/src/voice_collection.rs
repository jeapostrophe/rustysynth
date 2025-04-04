use crate::channel::Channel;
use crate::voice::Voice;

#[derive(Debug)]
pub(crate) struct VoiceCollection {
    voices: Vec<Voice>,
    pub(crate) active_voice_count: usize,
}

impl VoiceCollection {
    pub(crate) fn new() -> Self {
        // XXX don't preallocate and just use a normal vector
        let mut voices: Vec<Voice> = Vec::new();
        for _i in 0..crate::MAXIMUM_POLYPHONY {
            voices.push(Voice::new());
        }

        Self {
            voices,
            active_voice_count: 0,
        }
    }

    pub(crate) fn request_new(&mut self) -> &mut Voice {
        // If the number of active voices is less than the limit, use a free one.
        if (self.active_voice_count) < self.voices.len() {
            let i = self.active_voice_count;
            self.active_voice_count += 1;
            return &mut self.voices[i];
        }

        // Too many active voices...
        // Find one which has the lowest priority.
        let mut candidate: usize = 0;
        let mut lowest_priority = f32::MAX;
        for i in 0..self.active_voice_count {
            let voice = &self.voices[i];
            let priority = voice.get_priority();
            if priority < lowest_priority {
                lowest_priority = priority;
                candidate = i;
            } else if priority == lowest_priority {
                // Same priority...
                // The older one should be more suitable for reuse.
                if voice.voice_length > self.voices[candidate].voice_length {
                    candidate = i;
                }
            }
        }
        &mut self.voices[candidate]
    }

    pub(crate) fn process(&mut self, data: &[i16], channels: &[Channel]) {
        let mut i: usize = 0;

        loop {
            if i == self.active_voice_count {
                return;
            }

            let voice = &mut self.voices[i];
            let channel_info = &channels[voice.channel as usize];
            if voice.process(data, channel_info) {
                i += 1;
            } else {
                self.active_voice_count -= 1;
                self.voices.swap(i, self.active_voice_count);
            }
        }
    }

    pub(crate) fn get_active_voices(&mut self) -> &mut [Voice] {
        &mut self.voices[0..self.active_voice_count]
    }

    pub(crate) fn clear(&mut self) {
        self.active_voice_count = 0;
    }
}
