use eframe::{
    egui_glow,
    glow::{self, Context},
};
use std::marker::PhantomData;

pub trait Vertex: Default {}

pub struct VertexBuffer<'a, T: Vertex> {
    vertices: Vec<T>,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    gl: &'a glow::Context,
}

use glow::HasContext as _;

use crate::error::Error;

impl<'a, T: Vertex> VertexBuffer<'a, T> {
    pub fn new(nverts: usize, gl: &'a glow::Context) -> Result<Self, Error> {
        let vertices = vec![T::default()];
        let (vao, vbo) = unsafe {
            (
                gl.create_vertex_array().map_err(|e| Error::GLError(e))?,
                gl.create_buffer().map_err(|e| Error::GLError(e))?,
            )
        };
        Ok(VertexBuffer {
            vertices,
            vao,
            vbo,
            gl,
        })
    }

    fn alloc(&mut self) -> Result<Self, Error> {
        self.free();
        todo!()
    }

    fn free(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

impl<'a, T: Vertex> Drop for VertexBuffer<'a, T> {
    fn drop(&mut self) {
        self.free()
    }
}
