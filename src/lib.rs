mod editor;
mod params;

#[macro_use]
extern crate vst;

use baseview::WindowHandle;
use fundsp::hacker::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use params::{Parameter, Parameters};
use raw_window_handle::{
    HasRawWindowHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{convert::TryFrom, sync::Arc, time::Duration};
use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, HostCallback, Info, Plugin, PluginParameters};
use vst::prelude::*;
use vst::util::AtomicFloat;
use wmidi::{Channel, Note, Velocity};

pub struct PluginEditor {
    params: Arc<Parameters>,
    window_handle: Option<WindowHandle>,
    is_open: bool,
}

#[derive(Default)]
struct SynthVst {
    audio: Box<dyn AudioUnit64 + Send>,
    sample_rate: f32,
    parameters: Arc<Parameters>,
    time: Duration,
    note: Option<(Note, Velocity)>,
    enabled: bool,
    graph: Box<dyn AudioUnit64>,
    editor: Option<PluginEditor>,
}

impl Plugin for SynthVst {
    #[allow(clippy::precedence)]
    fn new(_host: HostCallback) -> Self {
        let Parameters {
            modulation,
            dirty,
            pan,
        } = Parameters::default();

        let freq = || tag(Tag::Freq as i64, 440.);
        let modulation = || tag(Tag::Modulation as i64, modulation.get() as f64);

        let offset = || tag(Tag::NoteOn as i64, 0.);
        let env = || offset() >> envelope2(|t, offset| downarc((t - offset) * 2.));

        let c = pulse();
        let params: Arc<Parameters> = Arc::new(Default::default());

        let audio_graph =
            freq() >> sine() * freq() * modulation() + freq() >> env() * sine() >> split::<U2>();

        Self {
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send>,
            parameters: Default::default(),
            note: None,
            time: Duration::from_millis(500),
            sample_rate: 41_000f32,
            enabled: false,
            graph: Box::new(c),
            editor: Some(PluginEditor {
                params,
                window_handle: None,
                is_open: false,
            }),
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "SynthVst".into(),
            vendor: "rusty".into(),
            version: 1,
            unique_id: 128956,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 2,
            ..Info::default()
        }
    }

    fn init(&mut self) {
        // Set up logs
        let log_folder = dirs::home_dir().unwrap().join("tmp");
        let _ = std::fs::create_dir(log_folder.clone());
        let Info {
            name,
            version,
            unique_id,
            ..
        } = self.get_info();
        let id_string = format!("{name}-{version}-{unique_id}-log.txt");
        let log_file = ::std::fs::File::create(log_folder.join(id_string))
            .expect("could not write to log file");
        let log_config = ::simplelog::ConfigBuilder::new()
            .set_time_to_local(true)
            .build();
        let _ = simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file);
        log_panics::init();
        log::info!("init");
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        self.time = Duration::default();
        self.audio.reset(Some(rate as f64));
    }

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_inputs, mut outputs) = buffer.split();
        if outputs.len() == 2 {
            let (left, right) = (outputs.get_mut(0), outputs.get_mut(1));
            // process by 64 sized blocks, which is the max for fundsp
            for (left_chunk, right_chunk) in left
                .chunks_mut(MAX_BUFFER_SIZE)
                .zip(right.chunks_mut(MAX_BUFFER_SIZE))
            {
                let mut left_buffer = [0f64; MAX_BUFFER_SIZE];
                let mut right_buffer = [0f64; MAX_BUFFER_SIZE];

                self.graph
                    .process(64, &[], &mut [&mut left_buffer, &mut right_buffer]);

                self.set_tag_with_param(Tag::Modulation, Parameter::Modulation);

                if let Some((note, ..)) = self.note {
                    self.set_tag(Tag::Freq, note.to_freq_f64())
                }

                if self.enabled {
                    self.time += Duration::from_secs_f32(MAX_BUFFER_SIZE as f32 / self.sample_rate);
                    self.audio.process(
                        MAX_BUFFER_SIZE,
                        &[],
                        &mut [&mut left_buffer, &mut right_buffer],
                    );
                }

                for (chunk, output) in left_chunk.iter_mut().zip(left_buffer.iter()) {
                    *chunk = *output as f32;
                }

                for (chunk, output) in right_chunk.iter_mut().zip(right_buffer.iter()) {
                    *chunk = *output as f32;
                }
            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.parameters) as Arc<dyn PluginParameters>
    }

    fn process_events(&mut self, events: &vst::api::Events) {
        for event in events.events() {
            if let vst::event::Event::Midi(midi) = event {
                if let Ok(midi) = wmidi::MidiMessage::try_from(midi.data.as_slice()) {
                    match midi {
                        wmidi::MidiMessage::NoteOn(_channel, note, _velocity) => {
                            self.set_tag(Tag::NoteOn, self.time.as_secs_f64());
                            self.note = Some((note, _velocity));
                            self.enabled = true;
                        }
                        wmidi::MidiMessage::NoteOff(_channel, note, _velocity) => {
                            if let Some((current_note, ..)) = self.note {
                                if current_note == note {
                                    self.note = None;
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    fn start_process(&mut self) {
        if self.parameters.dirty.swap(false, Ordering::Relaxed) {
            self.update_audio_graph();
        }
    }
}

impl SynthVst {
    #[inline(always)]
    fn set_tag(&mut self, tag: Tag, value: f64) {
        self.audio.set(tag as i64, value);
    }

    #[inline(always)]
    fn set_tag_with_param(&mut self, tag: Tag, param: Parameter) {
        self.set_tag(tag, self.parameters.get_parameter(param as i32) as f64);
    }

    fn update_audio_graph(&mut self) {
        let pan = self.parameters.pan.get();

        let graph = lfo(move |t| {
            let pitch = 110.0;
            let duty = lerp11(0.01, 0.99, sin_hz(0.05, t));
            (pitch, duty)
        }) >> pulse()
            >> declick()
            >> hacker::pan((pan - 0.5f32) as f64);

        self.graph = Box::new(graph);
    }
}

#[derive(FromPrimitive, Clone, Copy)]
pub enum Tag {
    Freq = 0,
    Modulation = 1,
    NoteOn = 2,
}

pub struct VstParent(*mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0 as *mut ::std::ffi::c_void,
            ..MacOSHandle::empty()
        })
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::Win32WindowHandle;

        RawWindowHandle::Win32(Win32WindowHandle {
            hwnd: self.0,
            ..Win32WindowHandle::empty()
        })
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }
}

plugin_main!(SynthVst);

// fn construct_audio_graph() -> impl AudioNode {
//     // Pulse wave.
//     let c = lfo(|t| {
//         let pitch = 110.0;
//         let duty = lerp11(0.01, 0.99, sin_hz(0.05, t));
//         (pitch, duty)
//     }) >> pulse();
//     c
// }
