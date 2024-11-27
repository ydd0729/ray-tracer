use crate::rendering::graphics_api::wgpu::Wgpu;
use crate::rendering::wgpu::WgpuBindable;
use getset::Getters;
use wgpu::*;

#[derive(Getters)]
pub struct WgpuTexture<'a> {
    texture_outer: Option<&'a Texture>,
    texture_created: Option<Texture>,
    #[getset(get = "pub")]
    texture_view: TextureView,
    #[getset(get = "pub")]
    binding_type: BindingType,
    #[getset(get_copy = "pub")]
    binding_instruction: WgpuTextureBindingInstruction,
}

pub struct WgpuTextureBindingInstruction {
    pub visibility: ShaderStages,
    pub binding_type: WgpuTextureBindingType,
    pub storage_access: Option<StorageTextureAccess>,
    pub sample_type: Option<TextureSampleType>,
}

pub enum WgpuTextureBindingType {
    Texture,
    StorageTexture,
}

impl<'a> WgpuTexture<'a> {
    pub fn new(
        wgpu: &Wgpu,
        texture_descriptor: &TextureDescriptor,
        binding_instruction: WgpuTextureBindingInstruction,
    ) -> Self {
        let label = format!("{} texture", texture_descriptor.label.unwrap_or("default"));

        let texture = wgpu.device.create_texture(&TextureDescriptor {
            label: Some(&label),
            ..texture_descriptor.clone()
        });

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some(&label),
            ..Default::default()
        });

        let binding_type = Self::get_binding_type(texture.format(), &binding_instruction);

        Self {
            texture_outer: None,
            texture_created: Some(texture),
            texture_view,
            binding_type,
            binding_instruction,
        }
    }

    pub fn new_from_texture(
        label: &str,
        texture: &'a Texture,
        binding_instruction: WgpuTextureBindingInstruction,
    ) -> Self {
        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Label::from(format!("{} texture view", label).as_str()),
            ..Default::default()
        });

        let binding_type = Self::get_binding_type(texture.format(), &binding_instruction);

        Self {
            texture_outer: Some(texture),
            texture_created: None,
            texture_view,
            binding_type,
            binding_instruction,
        }
    }

    pub fn texture(&self) -> &Texture {
        if self.texture_created.is_some() {
            self.texture_created.as_ref().unwrap()
        } else {
            self.texture_outer.unwrap()
        }
    }

    pub fn set_binding_type(&mut self, binding_type: WgpuTextureBindingType) {
        self.binding_instruction.binding_type = binding_type;

        let texture = self.texture();
        self.binding_type = Self::get_binding_type(texture.format(), &self.binding_instruction)
    }

    fn get_binding_type(format: TextureFormat, binding_instruction: &WgpuTextureBindingInstruction) -> BindingType {
        match binding_instruction.binding_type {
            WgpuTextureBindingType::Texture => BindingType::Texture {
                sample_type: binding_instruction
                    .sample_type
                    .unwrap_or(TextureSampleType::Float { filterable: true }),
                view_dimension: Default::default(),
                multisampled: false,
            },
            WgpuTextureBindingType::StorageTexture => {
                BindingType::StorageTexture {
                    // WebGPU 只支持 Write ，其他 Access 需要 native only feature TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    access: binding_instruction
                        .storage_access
                        .unwrap_or(StorageTextureAccess::WriteOnly),
                    format,
                    view_dimension: Default::default(),
                }
            }
        }
    }
}

impl<'a> WgpuBindable<'a> for WgpuTexture<'_> {
    fn bind_group_layout_entry(&self) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: 0,
            visibility: self.binding_instruction.visibility,
            ty: self.binding_type,
            count: None,
        }
    }

    fn binding_resource(&'a self) -> BindingResource<'a> {
        BindingResource::TextureView(&self.texture_view)
    }
}
