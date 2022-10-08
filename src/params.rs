use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::Display;
use vst::prelude::*;

pub struct Parameters {
    pub freq: AtomicFloat,
    pub modulation: AtomicFloat,
}

#[derive(FromPrimitive, Clone, Copy)]
pub enum Parameter {
    Freq = 0,
    Modulation = 1,
}

impl Parameter {
    pub fn count() -> usize {
        [Parameter::Freq, Parameter::Modulation]
            .to_vec()
            .iter()
            .count()
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            freq: AtomicFloat::new(0.44),
            modulation: AtomicFloat::new(1.),
        }
    }
}

impl PluginParameters for Parameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match FromPrimitive::from_i32(index) {
            Some(Parameter::Freq) => self.freq.get(),
            Some(Parameter::Modulation) => self.modulation.get(),
            _ => 0f32,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match FromPrimitive::from_i32(index) {
            Some(Parameter::Freq) => self.freq.set(value),
            Some(Parameter::Modulation) => self.modulation.set(value),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        let param: Option<Parameter> = FromPrimitive::from_i32(index);
        param
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Parameter::Freq => "Frenquency",
                Parameter::Modulation => "Modulation",
            }
        )
    }
}
