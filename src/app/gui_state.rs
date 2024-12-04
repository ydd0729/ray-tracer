use crate::app::camera::CameraUpdateParameters;
use egui::{Color32, RichText, Ui};
use getset::{CopyGetters, Getters};

use super::renderer::RenderStatue;
use egui::special_emojis::GITHUB;

#[derive(Default, Getters, CopyGetters)]
pub struct GuiState {
    #[getset(get_copy = "pub")]
    pub samples_per_pixel: u32,
    #[getset(get_copy = "pub")]
    pub max_ray_bounces: u32,
    #[getset(get = "pub")]
    pub camera_update_parameters: CameraUpdateParameters,
    pub render_status: RenderStatue,
    pub progress: f32,
}

impl GuiState {
    pub fn new(samples_per_pixel: u32, max_ray_bounces: u32, camera_update_parameters: CameraUpdateParameters) -> Self {
        Self {
            samples_per_pixel,
            max_ray_bounces,
            camera_update_parameters,
            render_status: Default::default(),
            progress: 0.0,
        }
    }

    pub fn create_ui(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Stats").strong());
        ui.separator();
        egui::Grid::new("Stats").min_col_width(160.0).show(ui, |ui| {
            ui.label("Frames Per Second");
            ui.label(format!("{}", self.render_status.frames_per_second));
            ui.end_row();

            ui.label("Progress");

            const R0: f32 = 0.0;
            const R1: f32 = 92.0;
            const G0: f32 = 92.0;
            const G1: f32 = 128.0;
            const B0: f32 = 128.0;
            const B1: f32 = 0.0;

            let r = (R0 + (R1 - R0) * self.progress) as u8;
            let g = (G0 + (G1 - G0) * self.progress) as u8;
            let b = (B0 + (B1 - B0) * self.progress) as u8;

            ui.add(
                egui::ProgressBar::new(self.progress)
                    .show_percentage()
                    .fill(Color32::from_rgb(r, g, b)),
            );
            ui.end_row();

            ui.label("");
            ui.label(
                RichText::new(format!(
                    "{} / {}",
                    self.render_status.sampled_count, self.render_status.total_sample
                ))
                .small(),
            );
            ui.end_row();
        });

        ui.label(RichText::new("Camera").strong());
        ui.separator();

        egui::Grid::new("camera").min_col_width(160.0).show(ui, |ui| {
            ui.label("Vertical Field of View");
            ui.add(egui::Slider::new(&mut self.camera_update_parameters.vfov, 10.0..=90.0));
            ui.end_row();

            // ui.label("Focus Distance");
            // ui.add(egui::Slider::new(
            //     &mut self.camera_update_parameters.focus_distance,
            //     0.1..=10.0,
            // ));
            // ui.end_row();

            // ui.label("Defocus Angle");
            // ui.add(egui::Slider::new(&mut self.camera_update_parameters.defocus_angle, 0.0..=1.0));
            // ui.end_row();

            ui.label("Movement Speed");
            ui.add(egui::Slider::new(
                &mut self.camera_update_parameters.movement_speed,
                0.1..=5.0,
            ));
            ui.end_row();

            ui.label("Rotation Scale");
            ui.add(egui::Slider::new(
                &mut self.camera_update_parameters.rotation_scale,
                1.0..=10.0,
            ));
            ui.end_row();
        });

        ui.label(RichText::new("Sample").strong());
        ui.separator();

        egui::Grid::new("sampling").min_col_width(160.0).show(ui, |ui| {
            ui.label("Samples");
            ui.add(egui::Slider::new(&mut self.samples_per_pixel, 1..=50000));
            ui.end_row();

            ui.label("Max Ray Bounces");
            ui.add(egui::Slider::new(&mut self.max_ray_bounces, 0..=128));
            ui.end_row();
        });

        ui.label(RichText::new("About").strong());
        ui.separator();

        ui.label("This is a GPU-accelerated ray tracer implemented with wgpu. Ray calculations are carried out in a compute shader.");
        ui.add_space(12.0);
        ui.label("You can hold the right mouse button to enter fly-through mode. Use your mouse to rotate the camera, and WASDQE to move.");
        ui.add_space(12.0);
        ui.label(format!("{GITHUB} https://github.com/ydd0729/ray-tracer"));
        ui.add_space(1.0);
    }

    pub fn update(&mut self, render_status: RenderStatue) {
        self.render_status = render_status;
        self.progress = self.render_status.sampled_count as f32 / self.render_status.total_sample as f32;
    }
}
