use fundsp::hacker::*;
use num_derive::FromPrimitive;
use params::{Parameter, Parameters};
use rand::Rng;
use std::borrow::BorrowMut;
use std::sync::Arc;
use vst::buffer::AudioBuffer;
use vst::prelude::*;
use wmidi::{Channel, Note, Velocity};

mod basic_signal;
mod params;
mod rand_generator;
mod tag;

const FREQ_SCALAR: f64 = 1000.;

pub struct SynthVst {
    audio: Box<dyn AudioUnit64 + Send>,
    parameters: Arc<Parameters>,
    note: Option<(Note, Velocity)>,
    last_note: Option<(Note, Velocity)>,
}

#[derive(FromPrimitive, Clone, Copy)]
pub enum Tag {
    Freq = 0,
    Modulation = 1,
    NoteOn = 2,
}

enum PlayMode {
    Random,
    Basic,
    Other,
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
}

impl Plugin for SynthVst {
    fn new(host: HostCallback) -> Self {
        let Parameters { freq, modulation } = Parameters::default();
        let hz = freq.get() as f64 * FREQ_SCALAR;

        let freq = || tag(Parameter::Freq as i64, hz);
        let modulation = || tag(Parameter::Modulation as i64, modulation.get() as f64);

        let audio_graph =
            freq() >> sine() * freq() * modulation() + freq() >> sine() >> split::<U2>();

        Self {
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send>,
            parameters: Default::default(),
            note: None,
            last_note: None,
        }
    }

    // Plugin info
    fn get_info(&self) -> Info {
        Info {
            name: "SynthVst".into(),
            vendor: "rusty".into(),
            unique_id: 128956,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: Parameter::count() as i32,
            ..Info::default()
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.parameters) as Arc<dyn PluginParameters>
    }

    // Modify audio buffer
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        basic_signal::play(self, buffer);
    }

    fn process_events(&mut self, events: &vst::api::Events) {
        for event in events.events() {
            if let vst::event::Event::Midi(midi) = event {
                if let Ok(midi) = wmidi::MidiMessage::try_from(midi.data.as_slice()) {
                    match midi {
                        wmidi::MidiMessage::NoteOn(_channel, note, _velocity) => {
                            self.note = Some((note, _velocity));
                            if self.note == self.last_note || self.last_note == None {
                                self.last_note = Some((note, _velocity));
                            }
                        }
                        wmidi::MidiMessage::NoteOff(_channel, note, _velocity) => {
                            if let Some((current_note, ..)) = self.note {
                                if current_note == note {
                                    self.note = self.last_note;
                                } else {
                                    self.note = None;
                                    self.last_note = Some((note, _velocity));
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            //event.
        }
    }
}

// Build plugin
vst::plugin_main!(SynthVst);
