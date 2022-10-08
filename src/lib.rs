use fundsp::hacker::*;
use params::{Parameter, Parameters};
use rand::Rng;
use std::borrow::BorrowMut;
use std::sync::Arc;
use vst::buffer::AudioBuffer;
use vst::prelude::*;

mod basic_signal;
mod params;
mod rand_generator;

const FREQ_SCALAR: f64 = 1000.;

pub struct SynthVst {
    audio: Box<dyn AudioUnit64 + Send>,
    play_mode: PlayMode,
    parameters: Arc<Parameters>,
}

enum PlayMode {
    Random,
    Basic,
    Other,
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
            play_mode: PlayMode::Basic,
            parameters: Default::default(),
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
            parameters: 0,
            ..Info::default()
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.parameters) as Arc<dyn PluginParameters>
    }

    // Modify audio buffer
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        match self.play_mode {
            PlayMode::Random => rand_generator::play(buffer),
            PlayMode::Basic => basic_signal::play(self, buffer),
            _ => todo!(),
        }
    }
}

// Build plugin
vst::plugin_main!(SynthVst);
