use crate::app::camera::CameraUpdateParameters;
use egui::Ui;
use getset::{CopyGetters, Getters};

#[derive(Default, Getters, CopyGetters)]
pub struct GuiState {
    pub checkbox: bool,

    #[getset(get_copy = "pub")]
    pub samples_per_pixel: u32,

    #[getset(get_copy = "pub")]
    pub max_ray_bounces: u32,

    #[getset(get = "pub")]
    pub camera_update_parameters: CameraUpdateParameters,
}

impl GuiState {
    pub fn create_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.checkbox, "Show Panels");
    }
}
