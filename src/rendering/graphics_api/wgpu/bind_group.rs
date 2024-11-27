use crate::rendering::wgpu::Wgpu;
use getset::*;
use wgpu::*;

pub trait WgpuBindable<'a> {
    fn bind_group_layout_entry(&self) -> BindGroupLayoutEntry;
    fn binding_resource(&'a self) -> BindingResource<'a>;
}

#[derive(CopyGetters, Getters)]
pub struct WgpuBindGroup {
    // label: Option<&'a str>,
    #[getset(get_copy = "pub")]
    group_id: u32,

    #[getset(get = "pub")]
    bind_group_layout: BindGroupLayout,

    #[getset(get = "pub")]
    bind_group: BindGroup,
}

impl WgpuBindGroup {
    pub fn new<'a>(wgpu: &Wgpu, label: Option<&str>, group_id: u32, entries: &'a [&dyn WgpuBindable<'a>]) -> Self {
        let mut bind_group_layout_entries = Vec::new();
        let mut bind_group_entries = Vec::<BindGroupEntry>::new();

        for (i, entry) in entries.iter().enumerate() {
            let mut layout = entry.bind_group_layout_entry();
            layout.binding = i as u32;
            bind_group_layout_entries.push(layout);

            bind_group_entries.push(BindGroupEntry {
                binding: i as u32,
                resource: entry.binding_resource(),
            });
        }

        let label = label.unwrap_or("");

        let bind_group_layout = wgpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from(format!("{} bind group layout", label).as_str()),
            entries: bind_group_layout_entries.as_slice(),
        });

        let bind_group = wgpu.device.create_bind_group(&BindGroupDescriptor {
            label: Label::from(format!("{} bind group", label).as_str()),
            layout: &bind_group_layout,
            entries: bind_group_entries.as_slice(),
        });

        Self {
            group_id,
            bind_group,
            bind_group_layout,
        }
    }
}
