use fundsp::hacker::*;
use std::borrow::BorrowMut;
use vst::buffer::AudioBuffer;
use vst::prelude::*;

use crate::SynthVst;

pub fn play(synth_vst: &mut SynthVst, buffer: &mut AudioBuffer<f32>) {
    let (_, mut outputs) = buffer.split();

    if outputs.len() != 2 {
        return;
    }

    let (left, right) = (outputs.get_mut(0), outputs.get_mut(1));

    for (left_chunk, right_chunk) in left
        .chunks_mut(MAX_BUFFER_SIZE)
        .zip(right.chunks_mut(MAX_BUFFER_SIZE))
    {
        let mut left_buffer = [0f64; MAX_BUFFER_SIZE];
        let mut right_buffer = [0f64; MAX_BUFFER_SIZE];

        synth_vst.audio.process(
            MAX_BUFFER_SIZE,
            &[],
            &mut [&mut left_buffer, &mut right_buffer],
        );

        for (chunck, output) in left_chunk.iter_mut().zip(left_buffer.iter()) {
            *chunck = *output as f32;
        }

        for (chunck, output) in right_chunk.iter_mut().zip(right_buffer.iter()) {
            *chunck = *output as f32;
        }
    }
}
