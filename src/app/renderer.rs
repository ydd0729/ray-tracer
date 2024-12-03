use crate::app::camera::Camera;
use crate::app::egui_renderer::EguiRenderer;
use crate::app::gui_state::GuiState;
use crate::rendering::material::Dielectric;
use crate::rendering::material::DiffuseLight;
use crate::rendering::material::Lambertian;
use crate::rendering::material::MaterialList;
use crate::rendering::material::MaterialType;
use crate::rendering::primitive::sphere::SphereData;
use crate::rendering::primitive::*;
use crate::rendering::wgpu::*;
use crate::rendering::RenderContext;
use crate::RAY_TRACING_SHADER;
use bytemuck::offset_of;
use egui_winit::EventResponse;
use log::info;
use nalgebra::Point4;
use std::borrow::Cow;
use std::cell::{Ref, RefMut};
use std::cmp;
use std::mem;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use wgpu::*;
use winit::dpi::PhysicalSize;

pub struct Renderer {
    render_context: RenderContext,
    render_context_uniform_buffer: WgpuBindBuffer,
    primitives_storage_buffer: WgpuBindBuffer,
    important_indices_storage_buffer: WgpuBindBuffer,
    quads_storage_buffer: WgpuBindBuffer,
    spheres_storage_buffer: WgpuBindBuffer,
    lambertian_materials_storage_buffer: WgpuBindBuffer,
    diffuse_light_materials_storage_buffer: WgpuBindBuffer,
    dielectric_materials_storage_buffer: WgpuBindBuffer,
    pixel_color_storage_buffer: WgpuBindBuffer,
    egui_renderer: EguiRenderer,
    should_rerender: bool,
    samples_per_pixel: u32,
}

pub struct RendererParameters<'a> {
    pub samples_per_pixel: u32,
    pub max_ray_bounces: u32,
    pub max_width: u32,
    pub max_height: u32,
    #[allow(unused)]
    pub clear_color: Point4<f32>,
    pub window: Arc<winit::window::Window>,
    pub camera: Ref<'a, Camera>,
    pub primitives: &'a [Rc<Primitive>],
    pub important_indices: &'a [u32],
    pub materials: &'a MaterialList,
}

impl<'a> RendererParameters<'a> {
    pub fn max_pixels(&self) -> u32 {
        self.max_width * self.max_height
    }
}

impl Renderer {
    pub fn new(wgpu: Ref<Wgpu>, parameters: &RendererParameters) -> Self {
        let mut primitives_data = Vec::new();
        let mut quads_data = Vec::new();
        let mut spheres_data = Vec::new();

        for primitive in parameters.primitives.iter().map(Rc::as_ref) {
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
        primitives_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(primitives_data.as_slice()));

        let important_indices_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "important indices storage",
            (size_of::<PrimitiveData>() * cmp::max(parameters.important_indices.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        important_indices_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(parameters.important_indices));

        let quads_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "quad storage",
            (size_of::<QuadData>() * cmp::max(quads_data.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        quads_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(quads_data.as_slice()));

        let spheres_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "sphere storage",
            (size_of::<SphereData>() * cmp::max(spheres_data.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        spheres_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(spheres_data.as_slice()));

        let mut lambertian_materials = Vec::new();
        let mut diffuse_light_materials = Vec::new();
        let mut dielectric_materials = Vec::new();
        for (material_type, materials) in parameters.materials.map() {
            match material_type {
                MaterialType::DebugNormal => (),
                MaterialType::Lambertian => lambertian_materials.append(
                    &mut materials
                        .iter()
                        .map(|material| material.as_any().downcast_ref::<Lambertian>().unwrap().clone())
                        .collect(),
                ),
                MaterialType::DiffuseLight => diffuse_light_materials.append(
                    &mut materials
                        .iter()
                        .map(|material| material.as_any().downcast_ref::<DiffuseLight>().unwrap().clone())
                        .collect(),
                ),
                MaterialType::Dielectric => dielectric_materials.append(
                    &mut materials
                        .iter()
                        .map(|material| material.as_any().downcast_ref::<Dielectric>().unwrap().clone())
                        .collect(),
                ),
            }
        }
        let lambertian_materials_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "lambertian materials storage",
            (size_of::<SphereData>() * cmp::max(lambertian_materials.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        lambertian_materials_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(&lambertian_materials));

        let diffuse_light_materials_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "diffuse light materials storage",
            (size_of::<SphereData>() * cmp::max(diffuse_light_materials.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        diffuse_light_materials_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(&diffuse_light_materials));

        let dielectric_materials_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "dielectric materials storage",
            (size_of::<SphereData>() * cmp::max(dielectric_materials.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        dielectric_materials_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(&dielectric_materials));

        let pixel_color_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "pixel color storage",
            ((size_of::<f32>() * 3) as u32 * parameters.max_pixels()) as BufferAddress,
            BufferUsages::STORAGE,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            false,
        );

        let egui_renderer = EguiRenderer::new(&parameters.window, &wgpu.device, wgpu.surface_configuration.format);

        let (width, height) = parameters.window.inner_size().into();
        let render_context = RenderContext::new(
            &parameters.camera,
            width,
            height,
            parameters.samples_per_pixel,
            parameters.max_ray_bounces,
            parameters.important_indices.len() as u32,
        );

        let render_context_uniform_buffer = WgpuBindBuffer::new(
            &wgpu,
            "ray tracing context",
            size_of_val(&render_context) as BufferAddress,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            true,
        );
        render_context_uniform_buffer.write(&wgpu, 0, bytemuck::bytes_of(&render_context));

        Self {
            render_context,
            render_context_uniform_buffer,
            primitives_storage_buffer,
            important_indices_storage_buffer,
            quads_storage_buffer,
            spheres_storage_buffer,
            lambertian_materials_storage_buffer,
            diffuse_light_materials_storage_buffer,
            dielectric_materials_storage_buffer,
            pixel_color_storage_buffer,
            egui_renderer,
            samples_per_pixel: parameters.samples_per_pixel,
            should_rerender: false,
        }
    }

    pub fn render(&mut self, wgpu: Ref<Wgpu>, surface: WgpuTexture) {
        let mut encoder = wgpu.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        if self.take_rerender() {
            self.render_context.reset_sample_id();
            info!("{:?}", self.render_context.sample_id);
        } else if self.render_context.sample_id < self.render_context.samples_per_pixel {
            self.render_context.increment_sample_id();
            info!("{:?}", self.render_context.sample_id);
        }

        self.render_context_uniform_buffer.write(
            &wgpu,
            mem::offset_of!(RenderContext, sample_id),
            bytemuck::bytes_of(&self.render_context.sample_id),
        );

        let ray_tracing_bind_group = WgpuBindGroup::new(
            &wgpu,
            Option::from("ray tracing"),
            0,
            &[
                &self.render_context_uniform_buffer,
                &self.pixel_color_storage_buffer,
                &self.primitives_storage_buffer,
                &self.important_indices_storage_buffer,
                &self.quads_storage_buffer,
                &self.spheres_storage_buffer,
                &self.lambertian_materials_storage_buffer,
                &self.diffuse_light_materials_storage_buffer,
                &self.dielectric_materials_storage_buffer,
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

    pub fn on_resize(&mut self, wgpu: Ref<Wgpu>, size: &PhysicalSize<u32>, camera: Ref<Camera>) {
        self.render_context.update(&camera, size.width, size.height);
        self.render_context_uniform_buffer
            .write(&wgpu, 0, bytemuck::bytes_of(&self.render_context));
        self.should_rerender = true;
    }

    pub fn on_update(
        &mut self,
        window: Arc<winit::window::Window>,
        wgpu: Ref<Wgpu>,
        delta_time: Duration,
        mut camera: RefMut<Camera>,
        mut gui_state: RefMut<GuiState>,
    ) {
        if self.render_context.max_ray_bounces != gui_state.max_ray_bounces() {
            self.render_context.max_ray_bounces = gui_state.max_ray_bounces();
            self.render_context_uniform_buffer.write(
                &wgpu,
                offset_of!(RenderContext, max_ray_bounces),
                bytemuck::bytes_of(&self.render_context.max_ray_bounces),
            );
            self.should_rerender = true;
        }

        if self.samples_per_pixel != gui_state.samples_per_pixel() {
            self.samples_per_pixel = gui_state.samples_per_pixel();
            self.should_rerender = true;
        }

        if camera.take_rerender() {
            self.should_rerender = true;
        }

        let (width, height) = window.inner_size().into();
        self.render_context.update(&camera, width, height);
        self.render_context_uniform_buffer
            .write(&wgpu, 0, bytemuck::bytes_of(&self.render_context));

        self.egui_renderer.update(&window, delta_time, gui_state.deref_mut())
    }

    pub fn on_window_event(
        &mut self,
        window: Arc<winit::window::Window>,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.egui_renderer.on_window_event(&window, event)
    }

    fn take_rerender(&mut self) -> bool {
        if self.should_rerender {
            self.should_rerender = false;
            return true;
        }
        false
    }
}
