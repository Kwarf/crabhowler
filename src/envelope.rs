use clack_plugin::events::event_types::ParamValueEvent;

#[derive(Clone)]
pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.8,
            release: 0.1,
        }
    }
}

impl Envelope {
    pub fn handle_event(&mut self, event: &ParamValueEvent) {
        match event.param_id().map(|x| x.into()) {
            Some(0) => self.attack = event.value() as f32,
            Some(1) => self.decay = event.value() as f32,
            Some(2) => self.sustain = event.value() as f32,
            Some(3) => self.release = event.value() as f32,
            _ => {}
        }
    }
}
