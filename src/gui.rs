mod egui;
mod state;

use crate::gui::egui::EguiRenderer;
use crate::gui::state::GuiState;
use crate::rendering::wgpu::Wgpu;
use crate::time;
use egui_winit::EventResponse;
use std::cell::Ref;
use std::sync::Arc;
use wgpu::{Device, TextureFormat};

pub(crate) struct Gui {
    renderer: EguiRenderer,
    state: GuiState,
}

impl Gui {
    pub fn new(window: &winit::window::Window, device: &Device, surface_format: TextureFormat) -> Self {
        Self {
            renderer: EguiRenderer::new(window, device, surface_format),
            state: Default::default(),
        }
    }

    pub fn on_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.renderer.on_window_event(window, event)
    }

    pub fn update(&mut self, window: Arc<winit::window::Window>, delta_time: time::Duration) {
        self.renderer.update(window, delta_time, &mut self.state);
    }

    pub fn render(&mut self, wgpu: Ref<Wgpu>, surface_view: &wgpu::TextureView) {
        self.renderer.render(wgpu, surface_view)
    }
}
