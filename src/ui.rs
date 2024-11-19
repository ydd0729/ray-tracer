use crate::wgpu_context::WgpuContext;
use imgui::Condition;
use std::cell::Ref;
use std::sync::Arc;
use std::time;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;

pub(crate) struct Ui {
    im_gui: ImGuiContext,
}

impl Ui {
    pub(crate) fn new(
        window: &winit::window::Window,
        wgpu_context: &WgpuContext,
    ) -> Self {
        let im_gui = ImGuiContext::new(window, wgpu_context);
        Self { im_gui }
    }

    pub(crate) fn update(&mut self, window: Arc<winit::window::Window>, delta_time: &time::Duration) {
        self.im_gui.context.io_mut().update_delta_time(*delta_time);
        self.im_gui.platform
            .prepare_frame(self.im_gui.context.io_mut(), window.as_ref())
            .expect("Failed to prepare frame");
        let ui = self.im_gui.context.frame();
        {
            let gui_window = ui.window("Hello world");
            gui_window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Hello world!");
                    ui.text("This...is...imgui-rs on WGPU!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            let gui_window = ui.window("Hello too");
            gui_window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!("frame time: {delta_time:?}"));
                });

            if self.im_gui.last_cursor != ui.mouse_cursor() {
                self.im_gui.last_cursor = ui.mouse_cursor();
                self.im_gui.platform.prepare_render(ui, window.as_ref());
            }

            let mut show_demo_window = true;
            ui.show_demo_window(&mut show_demo_window);
        }
    }

    pub(crate) fn window_event(
        &mut self,
        window: Arc<winit::window::Window>,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.im_gui.platform.handle_event::<()>(
            self.im_gui.context.io_mut(),
            window.as_ref(),
            &Event::WindowEvent { window_id, event },
        );
    }

    pub(crate) fn render(
        &mut self,
        wgpu_context: Ref<WgpuContext>,
        surface_view: &wgpu::TextureView,
        clear: bool,
    ) {
        let mut encoder: wgpu::CommandEncoder = wgpu_context.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match clear {
                        false => wgpu::LoadOp::Load,
                        true => wgpu::LoadOp::Clear(wgpu_context.clear_color)
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.im_gui.renderer.render(
            self.im_gui.context.render(),
            &wgpu_context.queue,
            &wgpu_context.device,
            &mut rpass,
        ).expect("Rendering failed");

        drop(rpass);
        wgpu_context.queue.submit(Some(encoder.finish()));
    }
}

struct ImGuiContext {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    last_cursor: Option<imgui::MouseCursor>,
}

impl ImGuiContext {
    pub(crate) fn new(
        window: &winit::window::Window,
        wgpu_context: &WgpuContext,
    ) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::new(&mut context);
        platform.attach_window(
            context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
        context.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        context.fonts().add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = imgui_wgpu::RendererConfig {
            texture_format: wgpu_context.surface_configuration.format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(
            &mut context, &wgpu_context.device, &wgpu_context.queue, renderer_config,
        );
        let last_cursor = None;

        Self {
            context,
            platform,
            renderer,
            last_cursor,
        }
    }
}