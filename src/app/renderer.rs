use crate::app::camera::Camera;
use crate::app::egui_renderer::EguiRenderer;
use crate::app::gui_state::GuiState;
use crate::rendering::bvh::build_bvh_tree;
use crate::rendering::bvh::BvhBuildingEntry;
use crate::rendering::bvh::BvhNode;
use crate::rendering::material::*;
use crate::rendering::primitive::sphere::SphereData;
use crate::rendering::primitive::*;
use crate::rendering::wgpu::*;
use crate::rendering::RenderContext;
use crate::time;
use crate::RAY_TRACING_SHADER;
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
    bvh_storage_buffer: WgpuBindBuffer,
    important_indices_storage_buffer: WgpuBindBuffer,
    quads_storage_buffer: WgpuBindBuffer,
    spheres_storage_buffer: WgpuBindBuffer,
    lambertian_materials_storage_buffer: WgpuBindBuffer,
    diffuse_light_materials_storage_buffer: WgpuBindBuffer,
    dielectric_materials_storage_buffer: WgpuBindBuffer,
    pixel_color_storage_buffer: WgpuBindBuffer,
    egui_renderer: EguiRenderer,
    should_rerender: bool,
    frames_time: Option<time::Instant>,
    frames_count: u32,
    frames_per_second: u32,
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
    pub primitives: &'a [Rc<PrimitiveData>],
    pub important_indices: &'a [u32],
    pub materials: &'a MaterialList,
}

impl RendererParameters<'_> {
    pub fn max_pixels(&self) -> u32 {
        self.max_width * self.max_height
    }
}

#[derive(Default)]
pub struct RenderStatue {
    pub sampled_count: u32,
    pub total_sample: u32,
    pub frames_per_second: u32,
}

impl Renderer {
    pub fn new(wgpu: Ref<Wgpu>, parameters: &RendererParameters) -> Self {
        let mut primitives_indices = Vec::new();
        let mut quads_data = Vec::new();
        let mut spheres_data = Vec::new();
        let mut bvh_building = Vec::new();
        let mut importance = Vec::new();

        for primitive in parameters.primitives.iter().map(Rc::clone) {
            match primitive.as_ref() {
                PrimitiveData::Quad(quad) => {
                    bvh_building.push(BvhBuildingEntry {
                        primitive: Rc::clone(&primitive),
                        primitive_type: (*primitive).into(),
                        primitive_id: quads_data.len() as u32,
                        bounding_box: primitive.bounding_box(),
                    });
                    primitives_indices.push(PrimitiveIndex {
                        primitive_type: (*primitive).into(),
                        primitive_id: quads_data.len() as u32,
                    });
                    quads_data.push(*quad);
                }
                PrimitiveData::Sphere(sphere) => {
                    bvh_building.push(BvhBuildingEntry {
                        primitive: Rc::clone(&primitive),
                        primitive_type: (*primitive).into(),
                        primitive_id: spheres_data.len() as u32,
                        bounding_box: primitive.bounding_box(),
                    });
                    primitives_indices.push(PrimitiveIndex {
                        primitive_type: (*primitive).into(),
                        primitive_id: spheres_data.len() as u32,
                    });
                    spheres_data.push(*sphere);
                }
            }
        }

        for important in parameters.important_indices {
            importance.push(primitives_indices[*important as usize]);
        }

        let len = bvh_building.len();
        let mut bvh_tree = Vec::new();
        build_bvh_tree(&mut bvh_tree, &mut bvh_building, 0, len, 0);

        for (i, node) in bvh_tree.iter().enumerate() {
            info!("{} = {:?}\n", i, node);
        }
        let bvh_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "bvh storage",
            (size_of::<BvhNode>() * cmp::max(bvh_tree.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        bvh_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(bvh_tree.as_slice()));

        let important_indices_storage_buffer = WgpuBindBuffer::new(
            &wgpu,
            "important indices storage",
            (size_of::<PrimitiveIndex>() * cmp::max(importance.len(), 1)) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE,
            true,
        );
        important_indices_storage_buffer.write(&wgpu, 0, bytemuck::cast_slice(importance.as_slice()));

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
                        .map(|material| *material.as_any().downcast_ref::<Lambertian>().unwrap())
                        .collect(),
                ),
                MaterialType::DiffuseLight => diffuse_light_materials.append(
                    &mut materials
                        .iter()
                        .map(|material| *material.as_any().downcast_ref::<DiffuseLight>().unwrap())
                        .collect(),
                ),
                MaterialType::Dielectric => dielectric_materials.append(
                    &mut materials
                        .iter()
                        .map(|material| *material.as_any().downcast_ref::<Dielectric>().unwrap())
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
            bvh_storage_buffer,
            important_indices_storage_buffer,
            quads_storage_buffer,
            spheres_storage_buffer,
            lambertian_materials_storage_buffer,
            diffuse_light_materials_storage_buffer,
            dielectric_materials_storage_buffer,
            pixel_color_storage_buffer,
            egui_renderer,
            should_rerender: false,
            frames_time: None,
            frames_count: 0,
            frames_per_second: 0
        }
    }

    pub fn render(&mut self, wgpu: Ref<Wgpu>, surface: WgpuTexture) -> RenderStatue {
        self.frames_count += 1;

        let mut encoder = wgpu.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        if self.take_rerender() {
            self.render_context.reset_sample_id();
            // info!("{:?}", self.render_context.sample_id);
            self.frames_count = 0;
            self.frames_time = Some(time::Instant::now());
        } else if self.render_context.sample_id < self.render_context.samples_per_pixel {
            self.render_context.increment_sample_id();
            // info!("{:?}", self.render_context.sample_id);
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
                &self.bvh_storage_buffer,
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

        if self.frames_time.is_none() {
            self.frames_time = Some(time::Instant::now());
        } else {
            let elapsed = self.frames_time.as_ref().unwrap().elapsed().as_secs_f32();
            if elapsed > 0.5 {
                self.frames_per_second = (self.frames_count as f32 / elapsed).round() as u32;
                self.frames_time = Some(time::Instant::now());
                self.frames_count = 0;
            }
        }
        
        RenderStatue {
            sampled_count: self.render_context.sample_id,
            total_sample: self.render_context.samples_per_pixel,
            frames_per_second: self.frames_per_second,
        }
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
            self.should_rerender = true;
        }

        if self.render_context.samples_per_pixel != gui_state.samples_per_pixel() {
            self.render_context.set_samples_per_pixel(gui_state.samples_per_pixel());
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
