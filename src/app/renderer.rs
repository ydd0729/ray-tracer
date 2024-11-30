use crate::app::camera::Camera;
use crate::app::egui_renderer::EguiRenderer;
use crate::app::gui_state::GuiState;
use crate::rendering::primitive::sphere::SphereData;
use crate::rendering::primitive::*;
use crate::rendering::wgpu::*;
use crate::rendering::RenderContext;
use crate::RAY_TRACING_SHADER;
use egui_winit::EventResponse;
use nalgebra::Point4;
use std::borrow::Cow;
use std::cell::{Ref, RefMut};
use std::cmp;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use wgpu::*;
use winit::dpi::PhysicalSize;

pub struct Renderer {
    #[allow(unused)]
    max_ray_bounces: u32,
    #[allow(unused)]
    clear_color: Point4<f32>,

    render_context: RenderContext,
    render_context_uniform_buffer: WgpuBindBuffer,
    primitives_storage_buffer: WgpuBindBuffer,
    quads_storage_buffer: WgpuBindBuffer,
    spheres_storage_buffer: WgpuBindBuffer,
    pixel_color_storage_buffer: WgpuBindBuffer,
    egui_renderer: EguiRenderer,
}

#[derive(Default)]
pub struct RendererParameters {
    pub samples_per_pixel: u32,
    pub max_ray_bounces: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub clear_color: Point4<f32>,
}

impl RendererParameters {
    pub fn max_pixels(&self) -> u32 {
        self.max_width * self.max_height
    }
}

impl Renderer {
    pub fn new(
        wgpu: Ref<Wgpu>,
        window: Arc<winit::window::Window>,
        camera: &Camera,
        renderer_parameters: &RendererParameters,
        primitives: &[Rc<Primitive>],
    ) -> Self {
        let (width, height) = window.inner_size().into();
        let render_context = RenderContext::new(camera, width, height, renderer_parameters.samples_per_pixel);

        let render_context_uniform_buffer = WgpuBindBuffer::new(
            &wgpu,
            "ray tracing context",
            size_of_val(&render_context) as BufferAddress,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            true,
        );
        render_context_uniform_buffer.write(&wgpu, bytemuck::bytes_of(&render_context));

        let mut primitives_data = Vec::new();
        let mut quads_data = Vec::new();
        let mut spheres_data = Vec::new();

        for primitive in primitives.iter().map(Rc::as_ref) {
            match primitive {
                Primitive::Quad(quad) => {
                    primitives_data.push(PrimitiveData {
                        primitive_type: (*primitive).into(),
                        primitive_id: quads_data.len() as u32,
                    });
                    quads_data.push(*quad);
                }
                Primitive::Sphere(sphere) => {
                    primitives_data.push(PrimitiveData {
                        primitive_type: (*primitive).into(),
                        primitive_id: spheres_data.len() as u32,
                    });
                    spheres_data.push(*sphere);
                }
            }
        }
        let primitives_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "primitive storage",
            (size_of::<PrimitiveData>() * cmp::max(primitives_data.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        primitives_storage_buffer.write(&wgpu, bytemuck::cast_slice(primitives_data.as_slice()));

        let quads_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "quad storage",
            (size_of::<QuadData>() * cmp::max(quads_data.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        quads_storage_buffer.write(&wgpu, bytemuck::cast_slice(quads_data.as_slice()));

        let spheres_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "sphere storage",
            (size_of::<SphereData>() * cmp::max(spheres_data.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        spheres_storage_buffer.write(&wgpu, bytemuck::cast_slice(spheres_data.as_slice()));

        let pixel_color_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "pixel color storage",
            ((size_of::<f32>() * 3) as u32 * renderer_parameters.max_pixels()) as BufferAddress,
            BufferUsages::STORAGE,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            false,
        );

        let egui_renderer = EguiRenderer::new(&window, &wgpu.device, wgpu.surface_configuration.format);

        Self {
            max_ray_bounces: renderer_parameters.max_ray_bounces,
            clear_color: renderer_parameters.clear_color,
            render_context,
            render_context_uniform_buffer,
            primitives_storage_buffer,
            quads_storage_buffer,
            spheres_storage_buffer,
            pixel_color_storage_buffer,
            egui_renderer,
        }
    }

    pub fn render(&mut self, wgpu: Ref<Wgpu>, surface: WgpuTexture) {
        let mut encoder = wgpu.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let ray_tracing_bind_group = WgpuBindGroup::new(
            &wgpu,
            Option::from("ray tracing"),
            0,
            &[
                &self.render_context_uniform_buffer,
                &self.pixel_color_storage_buffer,
                &self.primitives_storage_buffer,
                &self.quads_storage_buffer,
                &self.spheres_storage_buffer,
                &surface,
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

        self.egui_renderer
            .render(&wgpu, &mut encoder, surface.texture_view(), None);

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
        wgpu: Ref<Wgpu>,
        delta_time: Duration,
        camera: &Camera,
        mut gui_state: RefMut<GuiState>,
    ) {
        let (width, height) = window.inner_size().into();
        self.render_context.update(camera, width, height);
        self.render_context_uniform_buffer
            .write(&wgpu, bytemuck::bytes_of(&self.render_context));

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
