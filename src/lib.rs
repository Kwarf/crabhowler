use clack_extensions::{
    audio_ports::{
        AudioPortFlags, AudioPortInfo, AudioPortInfoWriter, AudioPortType, PluginAudioPorts,
        PluginAudioPortsImpl,
    },
    note_ports::{
        NoteDialect, NoteDialects, NotePortInfo, NotePortInfoWriter, PluginNotePorts,
        PluginNotePortsImpl,
    },
    params::{
        ParamDisplayWriter, ParamInfo, ParamInfoFlags, ParamInfoWriter, PluginAudioProcessorParams,
        PluginMainThreadParams, PluginParams,
    },
    state::{PluginState, PluginStateImpl},
};
use clack_plugin::{
    clack_export_entry,
    entry::{DefaultPluginFactory, SinglePluginEntry},
    events::spaces::CoreEventSpace,
    host::{HostAudioProcessorHandle, HostMainThreadHandle, HostSharedHandle},
    plugin::{
        Plugin, PluginAudioProcessor, PluginDescriptor, PluginError, PluginMainThread, PluginShared,
    },
    prelude::{InputEvents, OutputEvents, PluginExtensions},
    process::{Audio, Events, PluginAudioConfiguration, Process, ProcessStatus},
    stream::{InputStream, OutputStream},
    utils::ClapId,
};
use envelope::Envelope;
use oscillator::{Oscillator, SineOscillator};
use std::{
    ffi::CStr,
    io::{Read, Write},
    sync::atomic::Ordering,
};

mod adsr;
mod envelope;
mod oscillator;

pub struct CrabHowler;

impl Plugin for CrabHowler {
    type AudioProcessor<'a> = CrabHowlerAudioProcessor<'a>;
    type Shared<'a> = CrabHowlerShared;
    type MainThread<'a> = CrabHowlerMainThread<'a>;

    fn declare_extensions(builder: &mut PluginExtensions<Self>, shared: Option<&Self::Shared<'_>>) {
        builder
            .register::<PluginAudioPorts>()
            .register::<PluginNotePorts>()
            .register::<PluginParams>()
            .register::<PluginState>();
    }
}

impl DefaultPluginFactory for CrabHowler {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("com.kwarf.crabhowler", "Crab Howler")
            .with_vendor("Kwarf")
            .with_features([INSTRUMENT, SYNTHESIZER, STEREO])
    }

    fn new_shared(_host: HostSharedHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(CrabHowlerShared::default())
    }

    fn new_main_thread<'a>(
        host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(Self::MainThread { shared })
    }
}

pub struct CrabHowlerAudioProcessor<'a> {
    osc: Box<dyn Oscillator + Send>,
    shared: &'a CrabHowlerShared,
}

impl<'a> PluginAudioProcessor<'a, CrabHowlerShared, CrabHowlerMainThread<'a>>
    for CrabHowlerAudioProcessor<'a>
{
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        main_thread: &mut CrabHowlerMainThread<'a>,
        shared: &'a CrabHowlerShared,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            osc: Box::new(SineOscillator::new(audio_config.sample_rate as f32)),
            shared,
        })
    }

    fn process(
        &mut self,
        process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        let mut output_port = audio
            .output_port(0)
            .ok_or(PluginError::Message("No output port"))?;

        let mut output_channels = output_port
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Output is not f32"))?;

        // A bit of acrobatics to get simultaneous mutable references to both the left and right channels
        let mut split = output_channels.split_at_mut(1);
        let (left, right) = (
            split
                .0
                .channel_mut(0)
                .ok_or(PluginError::Message("Left channel not found"))?,
            split
                .1
                .channel_mut(0)
                .ok_or(PluginError::Message("Right channel not found"))?,
        );

        for batch in events.input.batch() {
            for event in batch.events() {
                match event.as_core_event() {
                    Some(CoreEventSpace::NoteOn(event)) => {
                        self.osc.handle_note_on(&self.shared.envelope, event)
                    }
                    Some(CoreEventSpace::NoteOff(event)) => self.osc.handle_note_off(event),
                    Some(CoreEventSpace::ParamValue(event)) => {
                        self.shared.envelope.handle_event(event)
                    }
                    _ => {}
                }
            }

            let (left, right) = (
                &mut left[batch.sample_bounds()],
                &mut right[batch.sample_bounds()],
            );

            self.osc.process(left, right);
        }

        if self.osc.is_active() {
            Ok(ProcessStatus::Continue)
        } else {
            Ok(ProcessStatus::Sleep)
        }
    }
}

impl<'a> PluginAudioProcessorParams for CrabHowlerAudioProcessor<'a> {
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
        for event in input_parameter_changes {
            if let Some(CoreEventSpace::ParamValue(event)) = event.as_core_event() {
                self.shared.envelope.handle_event(event);
            }
        }
    }
}

#[derive(Default)]
pub struct CrabHowlerShared {
    envelope: Envelope,
}

impl<'a> PluginShared<'a> for CrabHowlerShared {}

pub struct CrabHowlerMainThread<'a> {
    shared: &'a CrabHowlerShared,
}

impl<'a> PluginMainThreadParams for CrabHowlerMainThread<'a> {
    fn count(&mut self) -> u32 {
        4
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        if let Some((name, default)) = match param_index {
            0 => Some(("Attack", Envelope::default().attack)),
            1 => Some(("Decay", Envelope::default().decay)),
            2 => Some(("Sustain", Envelope::default().sustain)),
            3 => Some(("Release", Envelope::default().release)),
            _ => None,
        } {
            info.set(&ParamInfo {
                id: param_index.into(),
                flags: ParamInfoFlags::IS_AUTOMATABLE,
                cookie: Default::default(),
                name: name.as_bytes(),
                module: b"",
                min_value: 0.0,
                max_value: 1.0,
                default_value: default.load(Ordering::Relaxed) as f64,
            });
        }
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        let envelope = &self.shared.envelope;
        match param_id.into() {
            0 => Some(envelope.attack.load(Ordering::Relaxed) as f64),
            1 => Some(envelope.decay.load(Ordering::Relaxed) as f64),
            2 => Some(envelope.sustain.load(Ordering::Relaxed) as f64),
            3 => Some(envelope.release.load(Ordering::Relaxed) as f64),
            _ => None,
        }
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result {
        use std::fmt::Write;
        match param_id.into() {
            0 | 1 | 3 => write!(writer, "{:.2} s", value),
            2 => write!(writer, "{:.2} %", value * 100f64),
            _ => Err(std::fmt::Error),
        }
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &CStr) -> Option<f64> {
        let scale = if param_id == 2 { 0.01 } else { 1.0 };
        let input = text.to_str().ok()?;
        let suffix_idx = input
            .find(|c: char| !c.is_numeric() && c != '.' && c != ',')
            .unwrap_or_else(|| input.len());
        input[..suffix_idx].parse().map(|v: f64| v * scale).ok()
    }

    fn flush(
        &mut self,
        input_parameter_changes: &clack_plugin::prelude::InputEvents,
        _output_parameter_changes: &mut clack_plugin::prelude::OutputEvents,
    ) {
        for event in input_parameter_changes {
            if let Some(CoreEventSpace::ParamValue(event)) = event.as_core_event() {
                self.shared.envelope.handle_event(event);
            }
        }
    }
}

impl<'a> PluginStateImpl for CrabHowlerMainThread<'a> {
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        let envelope = &self.shared.envelope;
        output.write_all(&envelope.attack.load(Ordering::Relaxed).to_le_bytes())?;
        output.write_all(&envelope.decay.load(Ordering::Relaxed).to_le_bytes())?;
        output.write_all(&envelope.sustain.load(Ordering::Relaxed).to_le_bytes())?;
        output.write_all(&envelope.release.load(Ordering::Relaxed).to_le_bytes())?;
        Ok(())
    }

    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError> {
        let envelope = &self.shared.envelope;
        let mut buf = [0; 4];
        input.read_exact(&mut buf)?;
        envelope
            .attack
            .store(f32::from_le_bytes(buf), Ordering::Relaxed);
        input.read_exact(&mut buf)?;
        envelope
            .decay
            .store(f32::from_le_bytes(buf), Ordering::Relaxed);
        input.read_exact(&mut buf)?;
        envelope
            .sustain
            .store(f32::from_le_bytes(buf), Ordering::Relaxed);
        input.read_exact(&mut buf)?;
        envelope
            .release
            .store(f32::from_le_bytes(buf), Ordering::Relaxed);
        Ok(())
    }
}

impl<'a> PluginAudioPortsImpl for CrabHowlerMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        if !is_input {
            1
        } else {
            0
        }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
        if !is_input && index == 0 {
            writer.set(&AudioPortInfo {
                id: ClapId::new(1),
                name: b"main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: None,
            });
        }
    }
}

impl<'a> PluginNotePortsImpl for CrabHowlerMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input {
            1
        } else {
            0
        }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut NotePortInfoWriter) {
        if is_input && index == 0 {
            writer.set(&NotePortInfo {
                id: ClapId::new(1),
                name: b"main",
                preferred_dialect: Some(NoteDialect::Clap),
                supported_dialects: NoteDialects::CLAP,
            })
        }
    }
}

impl<'a> PluginMainThread<'a, CrabHowlerShared> for CrabHowlerMainThread<'a> {}

clack_export_entry!(SinglePluginEntry<CrabHowler>);
