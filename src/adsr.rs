use crate::envelope::Envelope;

#[derive(PartialEq)]
pub enum ADSRState {
    Attack(f32),
    Decay(f32),
    Sustain,
    Release(f32),
    Ended,
}

pub struct ADSR {
    envelope: Envelope,
    pub state: ADSRState,
}

impl ADSR {
    pub fn new(envelope: Envelope) -> Self {
        ADSR {
            envelope,
            state: ADSRState::Attack(0.0),
        }
    }

    pub fn release(&mut self) {
        self.state = ADSRState::Release(0.0);
    }

    pub fn process(&mut self, sample_rate: f32) -> f32 {
        match self.state {
            ADSRState::Attack(sample) => {
                let attack_samples = self.envelope.attack * sample_rate;
                self.state = if sample >= attack_samples {
                    ADSRState::Decay(0.0)
                } else {
                    ADSRState::Attack(sample + 1.0)
                };
                sample / attack_samples
            }
            ADSRState::Decay(sample) => {
                let decay_samples = self.envelope.decay * sample_rate;
                self.state = if sample >= decay_samples {
                    ADSRState::Sustain
                } else {
                    ADSRState::Decay(sample + 1.0)
                };
                1.0 - (1.0 - self.envelope.sustain) * (sample / decay_samples)
            }
            ADSRState::Sustain => self.envelope.sustain,
            ADSRState::Release(sample) => {
                let release_samples = self.envelope.release * sample_rate;
                self.state = ADSRState::Release(sample + 1.0);
                self.state = if sample >= release_samples {
                    ADSRState::Ended
                } else {
                    ADSRState::Release(sample + 1.0)
                };
                self.envelope.sustain * (1.0 - sample / release_samples)
            }
            ADSRState::Ended => 0.0,
        }
    }
}
