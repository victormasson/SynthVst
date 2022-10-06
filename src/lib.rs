use rand::Rng;
use std::borrow::BorrowMut;
use vst::prelude::*;

struct SynthVst;

impl Plugin for SynthVst {
    fn new(host: HostCallback) -> Self {
        SynthVst
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
        let (_, mut outputs) = buffer.split();
        for output in outputs.borrow_mut() {
            rand::thread_rng().fill(output);
        }
    }
}

// Build plugin
vst::plugin_main!(SynthVst);
