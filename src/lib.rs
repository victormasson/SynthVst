use fundsp::hacker::*;
use rand::Rng;
use std::borrow::BorrowMut;
use vst::buffer::AudioBuffer;
use vst::prelude::*;

mod basic_signal;
mod rand_generator;

pub struct SynthVst {
    audio: Box<dyn AudioUnit64 + Send>,
    play_mode: PlayMode,
}

enum PlayMode {
    Random,
    Basic,
    Other,
}

impl Plugin for SynthVst {
    fn new(host: HostCallback) -> Self {
        let audio_graph = sine_hz(440.0) >> split::<U2>();
        Self {
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send>,
            play_mode: PlayMode::Basic,
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
