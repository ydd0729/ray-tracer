use crate::time;
use crate::wgpu_context::WgpuContext;
use egui::{ClippedPrimitive, FontData, FontDefinitions, FontFamily, TexturesDelta};
use egui_winit::EventResponse;
use std::cell::Ref;
use std::sync::Arc;
use wgpu::{Device, Label, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView};

pub(crate) struct UiRenderContext {
    screen_descriptor: egui_wgpu_backend::ScreenDescriptor,
    paint_jobs: Vec<ClippedPrimitive>,
    textures_delta: TexturesDelta,
    // delta_time: time::Duration,
}

pub(crate) struct Ui {
    e_gui_state: egui_winit::State,
    e_gui_render_pass: egui_wgpu_backend::RenderPass,
    checkbox: bool,
    ui_render_context: Option<UiRenderContext>,
    multisample: bool,
}

impl Ui {
    pub(crate) fn new(
        window: &winit::window::Window,
        device: &Device,
        surface_format: TextureFormat,
        multisample: bool,
    ) -> Self {
        let mut fonts = FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters):
        fonts.font_data.insert(
            "SourceHanSansCN-Medium".to_owned(),
            FontData::from_static(include_bytes!("../asset/font/SourceHanSansCN-Medium.otf")));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "SourceHanSansCN-Medium".to_owned());

        let gui_context = egui::Context::default();

        let scale_factor = window.scale_factor() as f32;
        gui_context.set_pixels_per_point(1.0);
        gui_context.set_fonts(fonts);

        let viewport_id = gui_context.viewport_id();
        let e_gui_state = egui_winit::State::new(
            gui_context,
            viewport_id,
            &window,
            Some(scale_factor),
            Some(winit::window::Theme::Dark),
            None,
        );

        Self {
            e_gui_state,
            e_gui_render_pass: egui_wgpu_backend::RenderPass::new(
                device,
                surface_format,
                // The number of samples calculated per pixel (for MSAA).
                // For non-multisampled textures, this should be 1
                if multisample { 4 } else { 1 },
            ),
            ui_render_context: None,
            checkbox: false,
            multisample,
        }
    }

    pub(crate) fn on_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.e_gui_state.on_window_event(window, event)
    }

    pub(crate) fn render_update(&mut self, window: Arc<winit::window::Window>, _delta_time: time::Duration) {
        let gui_input = self.e_gui_state.take_egui_input(window.as_ref());
        self.e_gui_state.egui_ctx().begin_pass(gui_input);

        egui::Window::new("Settings").show(self.e_gui_state.egui_ctx(), |ui| {
            ui.checkbox(&mut self.checkbox, "Show Panels");
        });

        let egui::FullOutput {
            textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = self.e_gui_state.egui_ctx().end_pass();

        let paint_jobs = self.e_gui_state.egui_ctx().tessellate(shapes, pixels_per_point);

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

        self.ui_render_context = Some(UiRenderContext {
            screen_descriptor,
            paint_jobs,
            textures_delta,
            // delta_time,
        })
    }

    pub(crate) fn render(
        &mut self,
        wgpu_context: Ref<WgpuContext>,
        surface_texture: &wgpu::SurfaceTexture,
        surface_view: &wgpu::TextureView,
    ) {
        if self.ui_render_context.is_none() {
            return;
        }

        let mut view = surface_view;
        let multisample_texture: Texture;
        let multisample_view: TextureView;

        if self.multisample {
            multisample_texture = wgpu_context.device.create_texture(&TextureDescriptor {
                label: Label::from("multisample_texture"),
                size: surface_texture.texture.size(),
                mip_level_count: 1,
                sample_count: 4,
                dimension: TextureDimension::D2,
                format: surface_texture.texture.format(),
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[surface_texture.texture.format()],
            });

            multisample_view = multisample_texture.create_view(&Default::default());
            view = &multisample_view;
        }

        let ui_render_context = self.ui_render_context.take().unwrap();

        self.e_gui_render_pass.add_textures(
            &wgpu_context.device, &wgpu_context.queue, &ui_render_context.textures_delta).expect("panic");
        self.e_gui_render_pass.remove_textures(ui_render_context.textures_delta).expect("panic");

        let mut encoder = wgpu_context.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.e_gui_render_pass.update_buffers(
            &wgpu_context.device,
            &wgpu_context.queue,
            &ui_render_context.paint_jobs,
            &ui_render_context.screen_descriptor,
        );

        self.e_gui_render_pass.execute(
            &mut encoder,
            view,
            &ui_render_context.paint_jobs,
            &ui_render_context.screen_descriptor,
            None,
        ).expect("panic");

        if self.multisample {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: Some(surface_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui main render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            drop(render_pass);
        }

        wgpu_context.queue.submit(Some(encoder.finish()));
    }
}