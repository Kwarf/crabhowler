use std::sync::atomic::Ordering;

use atomic_float::AtomicF32;
use clack_plugin::events::event_types::ParamValueEvent;

pub struct Envelope {
    pub attack: AtomicF32,
    pub decay: AtomicF32,
    pub sustain: AtomicF32,
    pub release: AtomicF32,
}

impl Clone for Envelope {
    fn clone(&self) -> Self {
        Self {
            attack: AtomicF32::new(self.attack.load(Ordering::Relaxed)),
            decay: AtomicF32::new(self.decay.load(Ordering::Relaxed)),
            sustain: AtomicF32::new(self.sustain.load(Ordering::Relaxed)),
            release: AtomicF32::new(self.release.load(Ordering::Relaxed)),
        }
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: AtomicF32::new(0.01),
            decay: AtomicF32::new(0.1),
            sustain: AtomicF32::new(0.8),
            release: AtomicF32::new(0.1),
        }
    }
}

impl Envelope {
    pub fn handle_event(&self, event: &ParamValueEvent) {
        match event.param_id().map(|x| x.into()) {
            Some(0) => self.attack.store(event.value() as f32, Ordering::Relaxed),
            Some(1) => self.decay.store(event.value() as f32, Ordering::Relaxed),
            Some(2) => self.sustain.store(event.value() as f32, Ordering::Relaxed),
            Some(3) => self.release.store(event.value() as f32, Ordering::Relaxed),
            _ => {}
        }
    }
}
