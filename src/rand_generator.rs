use rand::Rng;
use std::borrow::BorrowMut;
use vst::buffer::AudioBuffer;

pub fn play(buffer: &mut AudioBuffer<f32>) {
    let (_, mut outputs) = buffer.split();
    for output in outputs.borrow_mut() {
        rand::thread_rng().fill(output);
    }
}
