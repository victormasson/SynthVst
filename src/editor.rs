use std::sync::Arc;

use crate::{
    params::{Parameter, Parameters},
    PluginEditor, VstParent,
};
use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use egui::*;
use egui_baseview::{EguiWindow, Queue};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32WindowHandle};
use vst::{editor::Editor, plugin::PluginParameters};

const WINDOW_WIDTH: usize = 512;
const WINDOW_HEIGHT: usize = 512;

impl Editor for PluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        log::info!("Editor open");
        match self.is_open {
            true => false,
            false => {
                // ---------------------------- //
                // 4. Setting up `egui` for use //
                // ---------------------------- //
                self.is_open = true;
                let settings = WindowOpenOptions {
                    title: String::from("SynthVst"),
                    size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                    scale: WindowScalePolicy::SystemScaleFactor,
                };

                let window_handle = EguiWindow::open_parented(
                    &VstParent(parent),
                    settings,
                    self.params.clone(),
                    |_egui_ctx, _queue, _state| {},
                    |egui_ctx: &Context, _, state: &mut Arc<Parameters>| {
                        draw_ui(egui_ctx, state);
                    },
                );

                self.window_handle = Some(window_handle);
                true
            }
        }

        // let window_handle = EguiWindow::open_parented(
        //     &VstParent(parent),
        //     settings,
        //     self.params.clone(),
        //     |_egui_ctx: &CtxRef, _queue: &mut Queue, _state: &mut Arc<Parameters>| {},
        //     |egui_ctx: &CtxRef, _queue: &mut Queue, state: &mut Arc<Parameters>| {
        //         egui::Window::new("SynthVst").show(&egui_ctx, |ui| {
        //             let mut pan = state.pan.get();
        //             let mut modulation = state.modulation.get();
        //             if ui
        //                 .add(egui::Slider::new(&mut pan, 0.0..=1.0).text("Pan"))
        //                 .changed()
        //             {
        //                 state.pan.set(pan);
        //             }
        //             if ui
        //                 .add(egui::Slider::new(&mut modulation, 0.0..=10.0).text("Modulation"))
        //                 .changed()
        //             {
        //                 state.modulation.set(modulation);
        //             }
        //         });
        //     },
        // );

        // self.window_handle = Some(window_handle);

        // true
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
        if let Some(mut window_handle) = self.window_handle.take() {
            window_handle.close();
        }
    }
}

unsafe impl Send for VstParent {}

pub struct WindowParent(pub Win32WindowHandle);
unsafe impl Send for WindowParent {}

#[inline(always)]
fn draw_ui(ctx: &Context, state: &mut Arc<Parameters>) -> egui::Response {
    egui::CentralPanel::default()
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("hello rust");

                ui.label(format!(
                    "Pan: {}",
                    state.get_parameter(Parameter::Pan as i32)
                ));
                ui.label(format!(
                    "Modulation: {}",
                    state.get_parameter(Parameter::Modulation as i32)
                ));

                let mut pan = state.pan.get();
                let mut modulation = state.modulation.get();

                if ui
                    .add(egui::Slider::new(&mut pan, 0.0..=1.0).text("Pan"))
                    .changed()
                {
                    state.pan.set(pan);
                }

                if ui
                    .add(egui::Slider::new(&mut modulation, 0.0..=1.0).text("Modulation"))
                    .changed()
                {
                    state.modulation.set(modulation);
                }
            })
        })
        .response
}
