use std::sync::{Arc, RwLock};

use baseview::{Size, WindowHandle, WindowOpenOptions, WindowScalePolicy};
use clack_plugin::plugin::PluginError;
use egui_baseview::{
    egui::{self, Context, Slider},
    EguiWindow, GraphicsConfig, Queue,
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use crate::{envelope::Envelope, CrabHowlerShared};

pub struct CrabHowlerGui {
    pub parent: Option<RawWindowHandle>,
    handle: Option<WindowHandle>,
}

impl Default for CrabHowlerGui {
    fn default() -> Self {
        Self {
            parent: None,
            handle: None,
        }
    }
}

unsafe impl HasRawWindowHandle for CrabHowlerGui {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.parent.unwrap()
    }
}

impl CrabHowlerGui {
    pub fn open(&mut self, state: &CrabHowlerShared) -> Result<(), PluginError> {
        if self.parent.is_none() {
            return Err(PluginError::Message("No parent window provided"));
        }

        let settings = WindowOpenOptions {
            title: "CrabHowler".to_string(),
            size: Size::new(400.0, 200.0),
            scale: WindowScalePolicy::SystemScaleFactor,
            gl_config: Some(Default::default()),
        };

        self.handle = Some(EguiWindow::open_parented(
            self,
            settings,
            GraphicsConfig::default(),
            state.envelope.clone(),
            |_egui_ctx: &Context, _queue: &mut Queue, _state: &mut Arc<RwLock<Envelope>>| {},
            |egui_ctx: &Context, _queue: &mut Queue, state: &mut Arc<RwLock<Envelope>>| {
                let mut envelope = state.write().unwrap();

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.heading("Crab Howler");
                    ui.add(Slider::new(&mut envelope.attack, 0.0..=1.0).text("Attack"));
                    ui.add(Slider::new(&mut envelope.decay, 0.0..=1.0).text("Decay"));
                    ui.add(Slider::new(&mut envelope.sustain, 0.0..=1.0).text("Sustain"));
                    ui.add(Slider::new(&mut envelope.release, 0.0..=1.0).text("Release"));
                });
            },
        ));

        Ok(())
    }

    pub fn close(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            handle.close();
            self.handle = None;
        }
    }
}
