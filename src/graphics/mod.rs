/// The camera module holds the different camera options and helper functions for creating and
/// manipulating views.
pub mod camera;
/// The main Vulkan interface, holds backend components and
/// contextual information such as device, queue, and swapchain information.
pub mod context;
/// Holds all graphics error enums.
// pub mod error;
/// TODO: A module dedicated to images, used for textures and other image related things.
pub mod image;
/// The shader module defines types, traits, and structs to abstract complex operations that involve shaders.
/// This module has a lot of intense types from Vulkano wrapped in less scary interfaces that are not as troublesome to deal with
pub mod shader;

pub mod sprite;

use crate::graphics::context::GraphicsContext;
use vulkano::buffer::BufferAccess;

use cgmath::{prelude::Angle, Matrix, Matrix4, Rad, Vector3, Vector4};

use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::image::view::ImageViewAbstract;
use vulkano::sampler::Sampler;
use vulkano::descriptor_set::WriteDescriptorSet;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub enum BlendMode {
    Add,
    Subtract,
    Alpha,
    Invert,
    // Multiply,
    // Replace,
    // Lighten,
    // Darken,
}

pub trait Drawable {
    fn draw(&self, context: &mut GraphicsContext, info: DrawInfo);
}

pub struct PipelineData {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub vertex_count: u32,
    pub instance_buffer: Arc<CpuAccessibleBuffer<[InstanceData]>>,
    pub instance_count: u32,
    pub descriptors: Option<Vec<WriteDescriptorSet>>,
}

impl PipelineData {
    pub fn buffer(&mut self, binding: u32, buffer: Arc<dyn BufferAccess>) {
        if self.descriptors.is_none() {
            self.descriptors = Some(Vec::new())
        }

        self.descriptors.as_mut().unwrap().push(WriteDescriptorSet::buffer(binding, buffer))
    }

    pub fn sampled_image(
        &mut self, 
        binding: u32, 
        image_view: Arc<dyn ImageViewAbstract>, 
        sampler: Arc<Sampler>,
    ) {
        if self.descriptors.is_none() {
            self.descriptors = Some(Vec::new())
        }

        self.descriptors.as_mut().unwrap().push(
            WriteDescriptorSet::image_view_sampler(
                binding,
                image_view,
                sampler,
            ),
        )
    }
}

pub fn begin_frame(ctx: &mut GraphicsContext, color: Color) {
    ctx.begin_frame(color);
}

pub fn draw<D, T>(ctx: &mut GraphicsContext, drawable: &D, info: T)
where
    D: Drawable,
    T: Into<DrawInfo>,
{
    drawable.draw(ctx, info.into());
}

pub fn draw_with<D, T>(ctx: &mut GraphicsContext, drawable: &D, info: T, shader: shader::ShaderId)
where
    D: Drawable,
    T: Into<DrawInfo>,
{
    let prev_shader = ctx.current_shader.take().unwrap();
    ctx.current_shader = Rc::new(RefCell::new(Some(shader)));

    drawable.draw(ctx, info.into());
    
    ctx.current_shader = Rc::new(RefCell::new(Some(prev_shader)));
}

// TODO add result.
pub fn present(ctx: &mut GraphicsContext) {
    // let sleep_time = std::time::Duration::from_secs_f64(0.0166).checked_sub(ctx.last_frame_time.elapsed());
    // if let Some(value) = sleep_time { std::thread::sleep(value); }
    ctx.present();
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub vert_color: [f32; 4],
}

vulkano::impl_vertex!(Vertex, pos, uv, vert_color);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct InstanceData {
    src: [f32; 4],
    color: [f32; 4],
    transform: [[f32; 4]; 4],
}

impl From<DrawInfo> for InstanceData {
    fn from(info: DrawInfo) -> InstanceData {
        InstanceData {
            src: info.tex_rect.as_vec(),
            color: info.color.into(),
            transform: info.transform.as_mat4().into(),
        }
    }
}

const QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [0.0, 0.0, 0.0],
        uv: [0.0, 0.0],
        vert_color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0, 0.0],
        uv: [0.0, 1.0],
        vert_color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 0.0, 0.0],
        uv: [1.0, 0.0],
        vert_color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0, 0.0],
        uv: [1.0, 1.0],
        vert_color: [1.0, 1.0, 1.0, 1.0],
    },
];

vulkano::impl_vertex!(InstanceData, src, color, transform);

pub mod vs {
    vulkano_shaders::shader! { ty: "vertex", path: "src/graphics/shaders/texture.vert", }
}

pub mod fs {
    vulkano_shaders::shader! { ty: "fragment", path: "src/graphics/shaders/texture.frag", }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawInfo {
    pub tex_rect: Rect,
    pub color: Color,
    pub transform: Transform,
}

impl Default for DrawInfo {
    fn default() -> Self {
        Self {
            tex_rect: Rect::default(),
            color: Color::white(),
            transform: Transform::identity(),
        }
    }
}

impl DrawInfo {
    pub fn new() -> Self {
        Self {
            tex_rect: Rect::default(),
            color: Color::white(),
            transform: Transform::identity(),
        }
    }

    pub fn reset(&mut self) {
        self.tex_rect = Rect::default();
        self.color = Color::white();
        self.transform = Transform::identity();
    }

    pub fn with_rect(rect: Rect) -> Self {
        Self {
            tex_rect: rect,
            color: Color::white(),
            transform: Transform::identity(),
        }
    }

    pub fn with_transform(transform: Transform) -> Self {
        Self {
            tex_rect: Rect::default(),
            color: Color::white(),
            transform: transform,
        }
    }

    pub fn with_color(color: Color) -> Self {
        Self {
            tex_rect: Rect::default(),
            color: color,
            transform: Transform::identity(),
        }
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.transform.translate(x, y, z);
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.transform.rotate(x, y, z);
    }

    pub fn rotate_value(&mut self, r: f32) {
        self.transform.rotate_value(Rad(r));
    }

    pub fn nonuniform_scale(&mut self, x: f32, y: f32, z: f32) {
        self.transform.nonuniform_scale(x, y, z);
    }

    pub fn scale(&mut self, s: f32) {
        self.transform.nonuniform_scale(s, s, s);
    }

    pub fn dest(&mut self, x: f32, y: f32, z: f32) {
        self.transform.dest(x,y,z);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transform {
    Components {
        pos: Vector3<f32>,
        rotation: Rad<f32>,
        scale: Vector3<f32>,
        offset: Vector3<f32>,
    },
    Matrix(Matrix4<f32>),
}

impl Default for Transform {
    fn default() -> Self {
        Transform::identity()
    }
}

impl Transform {
    fn identity() -> Self {
        // Self::Matrix(Matrix4::identity())
        Self::Components {
            pos: Vector3::from((0.0, 0.0, 0.0)),
            rotation: Rad(0.0),
            scale: Vector3::from((1.0, 1.0, 1.0)),
            offset: Vector3::from((0.0, 0.0, 0.0)),
        }
    }

    pub fn as_mat4(&self) -> Matrix4<f32> {
        match self {
            Transform::Matrix(mat) => *mat,
            Transform::Components {
                pos,
                rotation,
                scale,
                offset,
            } => {
                let (sinr, cosr) = rotation.sin_cos();
                let cr00 = cosr * scale.x;
                let cr01 = -sinr * scale.y;
                let cr10 = sinr * scale.x;
                let cr11 = cosr * scale.y;
                let cr03 = offset.x * (1.0 - cr00) - offset.y * cr01 + pos.x;
                let cr13 = offset.y * (1.0 - cr11) - offset.x * cr10 + pos.y;

                Matrix4::from_cols(
                    Vector4::new(cr00, cr01, 0.0, cr03),
                    Vector4::new(cr10, cr11, 0.0, cr13),
                    Vector4::new(0.0, 0.0, 1.0, 0.0),
                    Vector4::new(0.0, 0.0, 0.0, 1.0),
                )
                .transpose()
            },
        }
    }

    fn dest(&mut self, x: f32, y: f32, z: f32) {
        match self {
            Transform::Matrix(_mat) => {
                // *mat = Matrix4::from_translation(Vector3::new(x, y, z)) * *mat;
            }
            Transform::Components { pos, .. } => {
                *pos = Vector3::from((x, y, z));
            }
        }
    }

    fn translate(&mut self, x: f32, y: f32, z: f32) {
        match self {
            Transform::Matrix(mat) => {
                *mat = Matrix4::from_translation(Vector3::new(x, y, z)) * *mat;
            }
            Transform::Components { pos, .. } => {
                *pos += Vector3::from((x, y, z));
            }
        }
    }

    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        let rotation = Matrix4::from_angle_x(Rad(x))
            + Matrix4::from_angle_y(Rad(y))
            + Matrix4::from_angle_z(Rad(z));
        match self {
            Transform::Matrix(mat) => {
                *mat = rotation * *mat;
            }
            Transform::Components {
                // rotation,
                ..
            } => {
                // *rotation += Rad(3.14);
            }
        }
    }

    fn rotate_value(&mut self, r: Rad<f32>) {
        match self {
            Transform::Matrix(_) => {}
            Transform::Components { rotation, .. } => {
                *rotation = r;
            }
        }
    }

    fn nonuniform_scale(&mut self, x: f32, y: f32, z: f32) {
        match self {
            Transform::Matrix(mat) => {
                println!("{:?}", Matrix4::from_nonuniform_scale(x, y, z));
                *mat = Matrix4::from_nonuniform_scale(x, y, z) * *mat;
            }
            Transform::Components { scale, .. } => {
                *scale = Vector3::from((x, y, z));
            }
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> [f32; 4] {
        color.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Color([f32; 4]);

impl From<[f32; 4]> for Color {
    fn from(a: [f32; 4]) -> Color {
        Color(a)
    }
}

impl Color {
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color([r as f32 / 255., g as f32 / 255., b as f32 / 255., a as f32 / 255.])
    }

    pub fn black() -> Color {
        Color([0.0, 0.0, 0.0, 1.0])
    }

    pub fn grey() -> Color {
        Color([0.25, 0.25, 0.25, 1.0])
    }

    pub fn white() -> Color {
        Color([1.0, 1.0, 1.0, 1.0])
    }

    pub fn red() -> Color {
        Color([1.0, 0.05, 0.05, 1.0])
    }

    pub fn transparent() -> Color {
        Color([0.0, 0.0, 0.0, 0.0])
    }

    pub fn as_u8_arr(&self) -> [u8; 4] {
        let mut arr = [0u8; 4];
        arr[0] = (self.0[0] * 255.) as u8;
        arr[1] = (self.0[1] * 255.) as u8;
        arr[2] = (self.0[2] * 255.) as u8;
        arr[3] = (self.0[3] * 255.) as u8;
        arr
    }

    pub fn as_u8_vec(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push((self.0[0] * 255.) as u8);
        v.push((self.0[1] * 255.) as u8);
        v.push((self.0[2] * 255.) as u8);
        v.push((self.0[3] * 255.) as u8);
        v
    }

    
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn as_vec(&self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }
}

impl Default for Rect {
    fn default() -> Self {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        }
    }
}
