use eframe::glow::{self, HasContext};

pub trait Vertex: Default + Copy + Clone + Sized + 'static {
    fn init_attributes(gl: &glow::Context);
}

pub struct VertexBuffer<T: Vertex> {
    vertices: Vec<T>,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

use crate::error::Error;

fn to_byte_slice<'a, T: Vertex>(data: &'a [T]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<T>()) }
}

impl<T: Vertex> VertexBuffer<T> {
    pub fn new(nverts: usize, gl: &glow::Context) -> Result<Self, Error> {
        let vertices = vec![T::default(); nverts];
        let (vao, vbo) = unsafe {
            (
                gl.create_vertex_array().map_err(|e| Error::GLError(e))?,
                gl.create_buffer().map_err(|e| Error::GLError(e))?,
            )
        };
        Ok(VertexBuffer { vertices, vao, vbo })
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    fn init(&mut self, gl: &glow::Context) -> Result<(), Error> {
        unsafe {
            self.vao = gl.create_vertex_array().map_err(|e| Error::GLError(e))?;
            self.vbo = gl.create_buffer().map_err(|e| Error::GLError(e))?;
        }
        Ok(())
    }

    fn alloc(&mut self, gl: &glow::Context) -> Result<(), Error> {
        self.free(gl);
        self.init(gl)?;
        // Bind and copy data.
        self.bind(gl);
        unsafe {
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                to_byte_slice(&self.vertices),
                glow::STATIC_DRAW,
            );
        }
        T::init_attributes(gl);
        self.unbind(gl);
        Ok(())
    }

    pub fn free(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
    }
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct MeshVertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    color: glam::Vec3,
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
