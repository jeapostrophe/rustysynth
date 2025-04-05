use crate::channel::Channel;
use crate::voice::Voice;

#[derive(Debug, Default)]
pub(crate) struct VoiceCollection(pub Vec<Voice>);

impl VoiceCollection {
    pub(crate) fn request_new(&mut self) -> &mut Voice {
        let voices = &mut self.0;
        let mut candidate: usize = voices.len();

        // If the number of active voices is less than the limit, use a free one.
        if candidate < crate::MAXIMUM_POLYPHONY {
            voices.push(Voice::default());
        } else {
            // Too many active voices...
            // Find one which has the lowest priority.
            let mut lowest_priority = f32::MAX;
            for i in 0..voices.len() {
                let voice = &voices[i];
                let priority = voice.get_priority();
                if priority < lowest_priority {
                    lowest_priority = priority;
                    candidate = i;
                } else if priority == lowest_priority {
                    // Same priority...
                    // The older one should be more suitable for reuse.
                    if voice.voice_length > voices[candidate].voice_length {
                        candidate = i;
                    }
                }
            }
        }
        &mut voices[candidate]
    }

    pub(crate) fn render(&mut self, data: &[i16], channels: &[Channel]) -> Vec<f32> {
        let mut output = vec![];
        self.0.retain_mut(|voice| {
            let channel_info = &channels[voice.channel as usize];
            match voice.render(data, channel_info) {
                Some(sample) => {
                    output.push(sample);
                    true
                }
                None => false,
            }
        });
        output
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}
