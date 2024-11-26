use crate::app::camera::Camera;
use crate::app::gui_state::GuiState;
use crate::rendering::egui_renderer::EguiRenderer;
use crate::rendering::primitive::Primitive::Quad;
use crate::rendering::primitive::{Primitive, PrimitiveQuad};
use crate::rendering::wgpu::{IWgpuBuffer, WgpuVertexBuffer};
use crate::rendering::wgpu::{Wgpu, WgpuRenderPass};
use crate::rendering::wgpu::{WgpuBindBuffer, WgpuBindGroup};
use crate::rendering::wgpu::{WgpuComputePass, WgpuIndexBuffer};
use crate::rendering::{RenderContext, Vertex};
use crate::{RAY_TRACING_SHADER, RESOLVE_SHADER};
use egui_winit::EventResponse;
use nalgebra::Point4;
use std::borrow::Cow;
use std::cell::{Ref, RefMut};
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use wgpu::*;
use winit::dpi::PhysicalSize;

pub struct Renderer {
    max_ray_bounces: u32,
    background_color: Point4<f32>,

    render_context: RenderContext,
    render_context_uniform_buffer: Rc<WgpuBindBuffer>,
    quads_storage_buffer: Rc<WgpuBindBuffer>,
    pixel_color_storage_buffer: Rc<WgpuBindBuffer>,
    resolve_vertex_buffer: WgpuVertexBuffer,
    resolve_index_buffer: WgpuIndexBuffer,

    egui_renderer: EguiRenderer,
}

#[derive(Default)]
pub struct RendererParameter {
    pub samples_per_pixel: u32,
    pub max_ray_bounces: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub background_color: Point4<f32>,
}

impl RendererParameter {
    pub fn max_pixels(&self) -> u32 {
        self.max_width * self.max_height
    }
}

impl Renderer {
    pub fn new(
        wgpu: Ref<Wgpu>,
        window: Arc<winit::window::Window>,
        camera: &Camera,
        gui_state: Ref<GuiState>,
        renderer_parameter: &RendererParameter,
        primitives: &Vec<Primitive>,
    ) -> Self {
        let window_size = window.inner_size();
        let width = window_size.width;
        let height = window_size.height;

        let render_context = RenderContext::new(camera, width, height, renderer_parameter.samples_per_pixel);

        let render_context_uniform_buffer = Rc::new(WgpuBindBuffer::new(
            &wgpu,
            "ray tracing context",
            size_of_val(&render_context) as BufferAddress,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            true,
        ));
        render_context_uniform_buffer.write(&wgpu, bytemuck::bytes_of(&render_context));

        let mut quads = Vec::new();
        for primitive in primitives {
            if let Quad(quad) = primitive {
                quads.push(*quad);
            }
        }
        let quads_storage_buffer = Rc::new(WgpuBindBuffer::new(
            &wgpu,
            "primitive storage",
            (size_of::<PrimitiveQuad>() * quads.len()) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        ));
        quads_storage_buffer.write(&wgpu, bytemuck::cast_slice(quads.as_slice()));

        let pixel_color_storage_buffer = Rc::new(WgpuBindBuffer::new(
            &wgpu,
            "pixel color storage",
            ((size_of::<f32>() * 3) as u32 * renderer_parameter.max_pixels()) as BufferAddress,
            BufferUsages::STORAGE,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            false,
        ));

        let resolve_vertices = [
            Vertex::default().with_position(-1.0, 1.0, 0.0, 1.0),
            Vertex::default().with_position(-1.0, -1.0, 0.0, 1.0),
            Vertex::default().with_position(1.0, -1.0, 0.0, 1.0),
            Vertex::default().with_position(1.0, 1.0, 0.0, 1.0),
        ];

        let resolve_index: [u32; 6] = [0, 1, 2, 2, 3, 0];

        let resolve_vertex_buffer = WgpuVertexBuffer::new(&wgpu, "resolve", resolve_vertices.len());
        resolve_vertex_buffer.write_vertex(&wgpu, &resolve_vertices);

        let resolve_index_buffer = WgpuIndexBuffer::new(&wgpu, "resolve", resolve_index.len());
        resolve_index_buffer.write_index(&wgpu, &resolve_index);

        let egui_renderer = EguiRenderer::new(&window, &wgpu.device, wgpu.surface_configuration.format);

        Self {
            max_ray_bounces: gui_state.max_ray_bounces(),
            background_color: renderer_parameter.background_color,
            render_context,
            render_context_uniform_buffer,
            quads_storage_buffer,
            pixel_color_storage_buffer,
            resolve_vertex_buffer,
            resolve_index_buffer,
            egui_renderer,
        }
    }

    pub fn render(&mut self, wgpu: Ref<Wgpu>, surface_view: &TextureView) {
        let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let ray_tracing_bind_group = WgpuBindGroup::new(
            &wgpu,
            Option::from("ray tracing"),
            0,
            vec![
                Rc::clone(&self.render_context_uniform_buffer),
                Rc::clone(&self.pixel_color_storage_buffer),
                Rc::clone(&self.quads_storage_buffer),
            ],
        );

        let ray_tracing_compute_pass = WgpuComputePass::new(
            &wgpu,
            "ray tracing",
            Some(&[ray_tracing_bind_group.bind_group_layout()]),
            &wgpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("ray tracing shader"),
                source: ShaderSource::Wgsl(Cow::Borrowed(*RAY_TRACING_SHADER)),
            }),
            [
                (self.render_context.width as f32 / 16f32).ceil() as u32,
                (self.render_context.height as f32 / 16f32).ceil() as u32,
                1,
            ],
        );

        ray_tracing_compute_pass.render(&mut encoder, Some(&[&ray_tracing_bind_group]));

        let resolve_bind_group = WgpuBindGroup::new(
            &wgpu,
            Option::from("resolve"),
            0,
            vec![
                Rc::clone(&self.render_context_uniform_buffer),
                Rc::clone(&self.pixel_color_storage_buffer),
            ],
        );

        let resolve_render_pass = WgpuRenderPass::new(
            &wgpu,
            "resolve",
            &[&self.resolve_vertex_buffer],
            Some(&[resolve_bind_group.bind_group_layout()]),
            &wgpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("resolve shader"),
                source: ShaderSource::Wgsl(Cow::Borrowed(*RESOLVE_SHADER)),
            }),
        );

        resolve_render_pass.render(
            &mut encoder,
            surface_view,
            &[&self.resolve_vertex_buffer],
            Some(&self.resolve_index_buffer),
            Some(&[&resolve_bind_group]),
            StoreOp::Store,
        );

        self.egui_renderer.render(&wgpu, &mut encoder, surface_view);

        wgpu.queue.submit(Some(encoder.finish()));
    }

    pub fn on_resize(&mut self, wgpu: Ref<Wgpu>, size: &PhysicalSize<u32>, camera: &Camera) {
        self.render_context.update(camera, size.width, size.height);
        self.render_context_uniform_buffer
            .write(&wgpu, bytemuck::bytes_of(&self.render_context));
    }

    pub fn on_update(
        &mut self,
        window: Arc<winit::window::Window>,
        delta_time: Duration,
        mut gui_state: RefMut<GuiState>,
    ) {
        self.egui_renderer.update(&window, delta_time, gui_state.deref_mut())
    }

    pub fn on_window_event(
        &mut self,
        window: Arc<winit::window::Window>,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.egui_renderer.on_window_event(&window, event)
    }
}
