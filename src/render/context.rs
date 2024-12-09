use crate::app::camera::Camera;
use crate::math::degree_to_radian;
use bytemuck::{Pod, Zeroable};
use log::info;
use nalgebra::{Point2, Point3, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod, Default, Debug)]
pub struct RenderContext {
    pub width: u32,
    pub height: u32,
    pub sample_position: Point2<u32>,
    pub pixel_origin: Point3<f32>, // Location of pixel 0, 0
    pub samples_per_pixel: u32,
    pub pixel_delta_u: Vector3<f32>, // Offset to pixel to the right
    pub sample_grid_num: u32,
    pub pixel_delta_v: Vector3<f32>, // Offset to pixel below
    pub defocus_angle: f32,
    pub defocus_disk_u: Vector3<f32>, // Defocus disk horizontal radius
    pub sample_grid_len: f32,
    pub defocus_disk_v: Vector3<f32>, // Defocus disk vertical radius
    pub sample_id: u32,
    pub camera_position: Point3<f32>,
    pub max_ray_bounces: u32,
    pub important_index_len: u32,
    pub samples_per_frame: u32,
    _padding: [u32; 2],
}

impl RenderContext {
    pub fn new(
        camera: &Camera,
        width: u32,
        height: u32,
        samples_per_pixel: u32,
        samples_per_frame: u32,
        max_ray_bounces: u32,
        important_index_len: u32,
    ) -> Self {
        let mut configuration = Self {
            max_ray_bounces,
            important_index_len,
            samples_per_frame,
            ..Default::default()
        };

        configuration.set_samples_per_pixel(samples_per_pixel);
        configuration.update(camera, width, height);

        info!("{:?}", configuration);

        configuration
    }

    pub fn set_samples_per_pixel(&mut self, samples_per_pixel: u32) {
        if samples_per_pixel < 1 {
            panic!("Samples per pixel must be greater than 0");
        }

        self.samples_per_pixel = samples_per_pixel;
        let sample_grid_per_dimension = self.sample_grid_per_dimension();
        self.sample_grid_num = sample_grid_per_dimension.pow(2);
        self.sample_grid_len = 1.0 / sample_grid_per_dimension as f32;
    }

    pub fn set_sample_id(&mut self, sample_id: u32) {
        self.sample_id = sample_id;
        let sample_grid_per_dimension = self.sample_grid_per_dimension();
        if sample_id < self.sample_grid_num {
            self.sample_position = Point2::new(
                sample_id % sample_grid_per_dimension,
                sample_id / sample_grid_per_dimension,
            );
        }
    }

    pub fn reset_sample_id(&mut self) {
        self.set_sample_id(0);
    }

    pub fn increment_sample_id(&mut self) {
        self.set_sample_id(self.sample_id + self.samples_per_frame);
    }

    fn sample_grid_per_dimension(&self) -> u32 {
        (self.samples_per_pixel as f32).sqrt().floor() as u32
    }

    pub fn update(&mut self, camera: &Camera, width: u32, height: u32) -> bool {
        let mut updated = false;
        
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            updated = true;
        }

        let theta = degree_to_radian(camera.vfov());
        let h = (theta / 2.0).tan();

        // 视口的高度和宽度
        let viewport_height = 2.0 * h * camera.focus_distance();
        let viewport_width = viewport_height * self.aspect_ratio();

        // 视口横向和纵向的方向和大小
        let viewport_u = camera.u().scale(viewport_width);
        let viewport_v = -camera.v().scale(viewport_height);

        let pixel_delta_u = viewport_u / self.width as f32;
        let pixel_delta_v = viewport_v / self.height as f32;

        if self.pixel_delta_u != pixel_delta_u || self.pixel_delta_v != pixel_delta_v {
            self.pixel_delta_u = pixel_delta_u;
            self.pixel_delta_v = pixel_delta_v;
            updated = true;
        }
        
        // 视口坐标的原点
        let viewport_origin =
            camera.position() - camera.w().scale(camera.focus_distance()) - (viewport_u + viewport_v) * 0.5;

        // 第 1 个像素的位置，与视口原点差 0.5 个像素长度
        let pixel_origin = viewport_origin + (self.pixel_delta_u + self.pixel_delta_v) * 0.5;
        let defocus_angle = camera.defocus_angle();
        
        if self.pixel_origin != pixel_origin || self.defocus_angle != defocus_angle {
            self.pixel_origin = pixel_origin;
            self.defocus_angle = defocus_angle;
            updated = true;
        }
        
        let defocus_radius = camera.focus_distance() * degree_to_radian(camera.defocus_angle() * 0.5).tan();
        let defocus_disk_u = camera.u().scale(defocus_radius);
        let defocus_disk_v = camera.v().scale(defocus_radius);
        
        if self.defocus_disk_u != defocus_disk_u || self.defocus_disk_v != defocus_disk_v {
            self.defocus_disk_u = defocus_disk_u;
            self.defocus_disk_v = defocus_disk_v;
            updated = true;
        }

        let camera_position = *camera.position();
        
        if self.camera_position != camera_position {
            self.camera_position = camera_position;
            updated = true;
        }
        
        updated
    }

    fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
    pub fn pixels(&self) -> u32 {
        self.width * self.height
    }
}
