use std::marker::PhantomData;

use eframe::glow::{self, HasContext};

pub trait Vertex: Default + Copy + Clone + Sized + 'static {
    fn init_attributes(gl: &glow::Context);
}

pub struct VertexBuffer<V: Vertex> {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    phantom: PhantomData<V>,
}

use crate::error::Error;

fn to_byte_slice<'a, T: Sized>(data: &'a [T]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<T>()) }
}

impl<V: Vertex> VertexBuffer<V> {
    pub fn from_iter<I>(iter: I, gl: &glow::Context) -> Result<Self, Error>
    where
        I: Iterator<Item = V>,
    {
        let vertices = iter.collect::<Vec<V>>();
        let (vao, vbo) = unsafe {
            (
                gl.create_vertex_array().map_err(|e| Error::GLError(e))?,
                gl.create_buffer().map_err(|e| Error::GLError(e))?,
            )
        };
        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                to_byte_slice(&vertices),
                glow::STATIC_DRAW,
            );
        }
        V::init_attributes(gl);
        // Unbind.
        unsafe {
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        Ok(VertexBuffer::<V> {
            vao,
            vbo,
            phantom: PhantomData,
        })
    }

    pub fn bind_vao(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
        }
    }

    pub fn free(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
    }
}

pub struct IndexBuffer {
    ibo: glow::Buffer,
    num_indices: usize,
}

impl IndexBuffer {
    pub fn from_iter<I>(iter: I, gl: &glow::Context) -> Result<IndexBuffer, Error>
    where
        I: Iterator<Item = u32>,
    {
        let indices: Vec<_> = iter.collect();
        let ibo = unsafe { gl.create_buffer().map_err(|e| Error::GLError(e))? };
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                to_byte_slice(&indices),
                glow::STATIC_DRAW,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
        Ok(IndexBuffer {
            ibo,
            num_indices: indices.len(),
        })
    }

    pub fn free(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.ibo);
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ibo));
        }
    }

    pub fn len(&self) -> usize {
        self.num_indices
    }
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct MeshVertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    color: glam::Vec3,
}

impl MeshVertex {
    pub fn new(position: glam::Vec3, normal: glam::Vec3, color: glam::Vec3) -> MeshVertex {
        MeshVertex {
            position,
            normal,
            color,
        }
    }
}

impl Vertex for MeshVertex {
    fn init_attributes(gl: &glow::Context) {
        const STRIDE: i32 = size_of::<MeshVertex>() as i32;
        const OFFSETS: (i32, i32, i32) = unsafe {
            let val = std::mem::MaybeUninit::uninit();
            let val: *const MeshVertex = val.as_ptr();
            let base = val as *const u8;
            let pos = std::ptr::addr_of!((*val).position) as *const u8;
            let normal = std::ptr::addr_of!((*val).normal) as *const u8;
            let color = std::ptr::addr_of!((*val).color) as *const u8;
            (
                pos.offset_from(base) as i32,
                normal.offset_from(base) as i32,
                color.offset_from(base) as i32,
            )
        };
        unsafe {
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, STRIDE, OFFSETS.0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, STRIDE, OFFSETS.1);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, STRIDE, OFFSETS.2);
            gl.enable_vertex_attrib_array(2);
        }
    }
}
