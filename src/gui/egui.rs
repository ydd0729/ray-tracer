use crate::gui::state::GuiState;
use crate::rendering::wgpu::Wgpu;
use crate::time;
use crate::{FONT_SOURCE_HANS_SANS_CN_MEDIUM, FONT_SOURCE_HANS_SANS_CN_MEDIUM_NAME};
use egui::TextStyle::{Body, Heading};
use egui::{ClippedPrimitive, FontData, FontDefinitions, FontFamily, Rounding, Shadow, TexturesDelta, Ui};
use egui_winit::EventResponse;
use std::cell::Ref;
use std::sync::Arc;
use wgpu::*;
use winit::window::{Theme, Window};

pub struct EguiRenderer {
    egui_state: egui_winit::State,
    egui_render_pass: egui_wgpu_backend::RenderPass,
    context: Option<EguiRenderingContext>,
}

struct EguiRenderingContext {
    screen_descriptor: egui_wgpu_backend::ScreenDescriptor,
    paint_jobs: Vec<ClippedPrimitive>,
    textures_delta: TexturesDelta,
}

impl EguiRenderer {
    pub fn new(window: &Window, device: &Device, surface_format: TextureFormat) -> Self {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            FONT_SOURCE_HANS_SANS_CN_MEDIUM_NAME.to_owned(),
            FontData::from_static(&FONT_SOURCE_HANS_SANS_CN_MEDIUM),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, FONT_SOURCE_HANS_SANS_CN_MEDIUM_NAME.to_owned());

        let gui_context = egui::Context::default();

        let scale_factor = window.scale_factor() as f32;
        gui_context.set_pixels_per_point(1.0);
        gui_context.set_fonts(fonts);

        // let text_styles: BTreeMap<_, _> = [
        //     (Heading, FontId::new(12.0, Proportional)),
        //     // (Name("Heading2".into()), FontId::new(25.0, Proportional)),
        //     // (Name("Context".into()), FontId::new(23.0, Proportional)),
        //     // (Body, FontId::new(18.0, Proportional)),
        //     // (Monospace, FontId::new(14.0, Proportional)),
        //     // (Button, FontId::new(14.0, Proportional)),
        //     // (Small, FontId::new(10.0, Proportional)),
        // ]
        // .into();
        gui_context.all_styles_mut(move |style| {
            let body_style = style.text_styles.get(&Body).unwrap().clone();
            let heading_style = style.text_styles.get_mut(&Heading).unwrap();
            heading_style.size = body_style.size;
        });

        let viewport_id = gui_context.viewport_id();
        let egui_state = egui_winit::State::new(
            gui_context,
            viewport_id,
            &window,
            Some(scale_factor),
            Some(Theme::Dark),
            None,
        );

        let egui_render_pass = egui_wgpu_backend::RenderPass::new(device, surface_format, 1);

        Self {
            egui_state,
            egui_render_pass,
            context: None,
        }
    }

    pub fn on_window_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> EventResponse {
        self.egui_state.on_window_event(window, event)
    }

    pub fn update(&mut self, window: Arc<Window>, _delta_time: time::Duration, gui_state: &mut GuiState) {
        let gui_input = self.egui_state.take_egui_input(window.as_ref());
        self.egui_state.egui_ctx().begin_pass(gui_input);

        self.egui_state.egui_ctx().set_visuals(egui::Visuals {
            window_shadow: Shadow::NONE,
            window_rounding: Rounding::same(8.0),
            ..Default::default()
        });

        let gui_window = egui::Window::new("Settings");
        gui_window.show(self.egui_state.egui_ctx(), |ui| Self::build_ui(ui, gui_state));

        let egui::FullOutput {
            textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = self.egui_state.egui_ctx().end_pass();

        let paint_jobs = self.egui_state.egui_ctx().tessellate(shapes, pixels_per_point);

        let screen_descriptor = {
            // let (width, height) = self.last_size;
            let width = window.inner_size().width;
            let height = window.inner_size().height;

            egui_wgpu_backend::ScreenDescriptor {
                physical_width: width,
                physical_height: height,
                scale_factor: pixels_per_point,
            }
        };

        self.context = Some(EguiRenderingContext {
            screen_descriptor,
            paint_jobs,
            textures_delta,
            // delta_time,
        })
    }

    pub fn render(&mut self, wgpu_context: Ref<Wgpu>, surface_view: &TextureView) {
        if self.context.is_none() {
            return;
        }

        let ui_render_context = self.context.take().unwrap();

        self.egui_render_pass
            .add_textures(
                &wgpu_context.device,
                &wgpu_context.queue,
                &ui_render_context.textures_delta,
            )
            .expect("panic");
        self.egui_render_pass
            .remove_textures(ui_render_context.textures_delta)
            .expect("panic");

        let mut encoder = wgpu_context.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.egui_render_pass.update_buffers(
            &wgpu_context.device,
            &wgpu_context.queue,
            &ui_render_context.paint_jobs,
            &ui_render_context.screen_descriptor,
        );

        self.egui_render_pass
            .execute(
                &mut encoder,
                surface_view,
                &ui_render_context.paint_jobs,
                &ui_render_context.screen_descriptor,
                None,
            )
            .expect("panic");

        wgpu_context.queue.submit(Some(encoder.finish()));
    }

    fn build_ui(ui: &mut Ui, state: &mut GuiState) {
        ui.checkbox(&mut state.checkbox, "Show Panels");
    }
}