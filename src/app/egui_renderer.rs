use crate::app::gui_state::GuiState;
use crate::rendering::wgpu::Wgpu;
use crate::{time, FONT_SOURCE_HANS_SANS_CN_MEDIUM, FONT_SOURCE_HANS_SANS_CN_MEDIUM_NAME};
use egui::{ClippedPrimitive, FontData, FontDefinitions, FontFamily, TexturesDelta};
use egui_winit::EventResponse;
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
        let gui_context = egui::Context::default();

        // Font
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

        gui_context.set_fonts(fonts);

        // scale
        gui_context.set_pixels_per_point(1.0);
        // gui_context.set_pixels_per_point(window.scale_factor() as f32);

        // Style
        // gui_context.all_styles_mut(move |style| {
        //     let body_style = style.text_styles.get_mut(&Button).unwrap();
        //     let heading_style = style.text_styles.get_mut(&Heading).unwrap();
        // });

        let viewport_id = gui_context.viewport_id();
        let egui_state = egui_winit::State::new(
            gui_context,
            viewport_id,
            &window,
            Some(window.scale_factor() as f32),
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

    pub fn update(&mut self, window: &Window, _delta_time: time::Duration, gui_state: &mut GuiState) {
        let gui_input = self.egui_state.take_egui_input(window);
        self.egui_state.egui_ctx().begin_pass(gui_input);

        self.egui_state.egui_ctx().set_visuals(egui::Visuals {
            // window_shadow: Shadow::NONE, // 移除窗口阴影
            // window_rounding: Rounding::same(8.0),
            window_highlight_topmost: false,
            ..Default::default()
        });

        let gui_window = egui::Window::new("Ray Tracer").default_width(288.0);
        gui_window.show(self.egui_state.egui_ctx(), |ui| gui_state.create_ui(ui));

        let egui::FullOutput {
            textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = self.egui_state.egui_ctx().end_pass();

        let paint_jobs = self.egui_state.egui_ctx().tessellate(shapes, pixels_per_point);

        let screen_descriptor = {
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
        })
    }

    pub fn render(
        &mut self,
        wgpu: &Wgpu,
        encoder: &mut CommandEncoder,
        surface_view: &TextureView,
        clear_color: Option<Color>,
    ) {
        if self.context.is_none() {
            return;
        }

        let ui_render_context = self.context.take().unwrap();

        self.egui_render_pass
            .add_textures(&wgpu.device, &wgpu.queue, &ui_render_context.textures_delta)
            .expect("panic");
        self.egui_render_pass
            .remove_textures(ui_render_context.textures_delta)
            .expect("panic");

        self.egui_render_pass.update_buffers(
            &wgpu.device,
            &wgpu.queue,
            &ui_render_context.paint_jobs,
            &ui_render_context.screen_descriptor,
        );

        self.egui_render_pass
            .execute(
                encoder,
                surface_view,
                &ui_render_context.paint_jobs,
                &ui_render_context.screen_descriptor,
                clear_color,
            )
            .expect("panic");
    }
}
