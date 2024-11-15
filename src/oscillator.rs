use clack_plugin::events::{
    event_types::{NoteOffEvent, NoteOnEvent},
    Match,
};

use crate::adsr::{ADSRState, ADSR};

pub trait Oscillator {
    fn handle_note_on(&mut self, envelope: &super::Envelope, event: &NoteOnEvent);
    fn handle_note_off(&mut self, event: &NoteOffEvent);
    fn process(&mut self, left: &mut [f32], right: &mut [f32]);
    fn is_active(&self) -> bool;
}

pub struct Voice {
    channel: u16,
    key: u16,
    note_id: Option<u32>,
    frequency: f32,
    phase: f32,
    velocity: f32,
    adsr: ADSR,
}

impl Voice {
    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        let increment = self.frequency / sample_rate;
        let sample = (self.phase * std::f32::consts::TAU).sin();
        self.phase = (self.phase + increment) % 1.0;
        sample
    }
}

pub struct SineOscillator {
    sample_rate: f32,
    voices: [Option<Voice>; 16],
}

impl SineOscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            voices: Default::default(),
        }
    }
}

impl Oscillator for SineOscillator {
    fn handle_note_on(&mut self, envelope: &super::Envelope, event: &NoteOnEvent) {
        if let (Match::Specific(channel), Match::Specific(key)) = (event.channel(), event.key()) {
            if let Some(voice) = self.voices.iter_mut().find(|voice| voice.is_none()) {
                *voice = Some(Voice {
                    channel,
                    key,
                    note_id: event.note_id().into_specific(),
                    frequency: 440.0 * 2.0f32.powf((key as f32 - 57.0) / 12.0),
                    phase: 0.0,
                    velocity: event.velocity() as f32,
                    adsr: ADSR::new(envelope.clone()),
                });
            }
        }
    }

    fn handle_note_off(&mut self, event: &NoteOffEvent) {
        if let Some(voice) = self.voices.iter_mut().flatten().find(|voice| {
            event.channel().as_specific() == Some(&voice.channel)
                && event.key().as_specific() == Some(&voice.key)
                && event.note_id().as_specific() == voice.note_id.as_ref()
        }) {
            voice.adsr.release();
        }
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32]) {
        left.fill(0.0);
        right.fill(0.0);

        // Turn off voices that have reached the end of their release phase
        self.voices
            .iter_mut()
            .filter(|voice| match voice {
                Some(voice) => voice.adsr.state == ADSRState::Ended,
                None => false,
            })
            .for_each(|voice| *voice = None);

        // Let's create a small vector to hold all active voices to (maybe?) increase performance a bit
        let mut active_voices = self.voices.iter_mut().flatten().collect::<Vec<_>>();

        for (left, right) in left.iter_mut().zip(right.iter_mut()) {
            for voice in &mut active_voices {
                let gain = 10f32.powf(voice.velocity * voice.adsr.process(self.sample_rate) - 1.0);
                let sample = voice.next_sample(self.sample_rate) * gain;

                *left += sample;
                *right += sample;
            }
        }
    }

    fn is_active(&self) -> bool {
        self.voices.iter().any(Option::is_some)
    }
}
