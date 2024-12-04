use bytemuck::{Pod, Zeroable};
use getset::Getters;
use nalgebra::Point3;
use std::{any::Any, collections::HashMap};

#[repr(u32)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum MaterialType {
    DebugNormal,
    Lambertian,
    DiffuseLight,
    Dielectric,
}

pub trait Material {
    fn material_type(&self) -> MaterialType;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone, Copy, Debug)]
pub struct MaterialHandle {
    pub material_type: u32,
    pub material_id: u32,
}

#[derive(Default, Getters)]
pub struct MaterialList {
    #[getset(get = "pub")]
    map: HashMap<MaterialType, Vec<Box<dyn Material>>>,
}

impl MaterialList {
    pub fn add(&mut self, material: Box<dyn Material>) -> MaterialHandle {
        self.map.entry(material.material_type()).or_default();
        let vec = self.map.get_mut(&material.material_type()).unwrap();
        let handler = MaterialHandle {
            material_type: material.material_type() as u32,
            material_id: vec.len() as u32,
        };
        vec.push(material);
        handler
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct DebugNormal {}

impl Material for DebugNormal {
    fn material_type(&self) -> MaterialType {
        MaterialType::DebugNormal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Lambertian {
    pub albedo: Point3<f32>,
    _padding: [u32; 1],
}

impl Lambertian {
    pub fn new(albedo: Point3<f32>) -> Self {
        Self {
            albedo,
            _padding: [0; 1],
        }
    }
}

impl Material for Lambertian {
    fn material_type(&self) -> MaterialType {
        MaterialType::Lambertian
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct DiffuseLight {
    pub emit: Point3<f32>,
    _padding: [u32; 1],
}

impl DiffuseLight {
    pub fn new(emit: Point3<f32>) -> Self {
        Self { emit, _padding: [0; 1] }
    }
}

impl Material for DiffuseLight {
    fn material_type(&self) -> MaterialType {
        MaterialType::DiffuseLight
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Dielectric {
    pub reflection_index: f32,
}

impl Dielectric {
    pub fn new(reflection_index: f32) -> Self {
        Self { reflection_index }
    }
}

impl Material for Dielectric {
    fn material_type(&self) -> MaterialType {
        MaterialType::Dielectric
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
