mod error;
mod meshview;

use alum::{element::Handle, mesh::PolyMeshF32};
use eframe::{egui, egui_glow, glow};
use egui::mutex::Mutex;
use meshview::{IndexBuffer, MeshVertex, VertexBuffer};
use std::sync::Arc;

macro_rules! gl_call {
    ($ctx:expr, $call:expr) => {{
        #[cfg(debug_assertions)]
        {
            let out = $call;
            let err: u32 = $ctx.get_error();
            if err != 0 {
                panic!("OpenGL Error {}", err);
            }
            out
        }
        #[cfg(not(debug_assertions))]
        {
            $call
        }
    }};
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        multisampling: 8,
        renderer: eframe::Renderer::Glow,
        depth_buffer: 24,
        follow_system_theme: true,
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using glow",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

const VIEWPORT_WIDTH: f32 = 2400.;
const VIEWPORT_HEIGHT: f32 = 1200.;
const FOVY: f32 = 0.9;
const NEAR: f32 = 0.01;
const FAR: f32 = 100.0;

struct MyApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    rotating_triangle: Arc<Mutex<Scene>>,
    projection: glam::Mat4,
    view: glam::Mat4,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        Self {
            rotating_triangle: Arc::new(Mutex::new(Scene::new(gl))),
            projection: glam::Mat4::perspective_rh_gl(
                FOVY,
                VIEWPORT_WIDTH / VIEWPORT_HEIGHT,
                NEAR,
                FAR,
            ),
            view: glam::Mat4::look_at_rh(
                glam::vec3(1.5, 1.5, 1.5),
                glam::vec3(0.0, 0.0, 0.0),
                glam::vec3(0.0, 0.0, 1.0),
            ),
        }
    }

    fn mvp_mat(&self) -> glam::Mat4 {
        self.projection * self.view
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The box is being painted using ");
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
            ui.label("Drag to rotate!");
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotating_triangle.lock().destroy(gl);
        }
    }
}

impl MyApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
            egui::Sense::drag(),
        );
        const FACTOR: f32 = 0.001;
        // let rot = glam::Mat4::from_rotation_x(response.drag_motion().x * FACTOR)
        //     * glam::Mat4::from_rotation_y(response.drag_motion().y * FACTOR);
        let inv = self.view.inverse();
        let x = inv * glam::vec4(0., 1., 0., 0.);
        let y = inv * glam::vec4(1., 0., 0., 0.);
        let rot = glam::Quat::from_axis_angle(x.truncate(), response.drag_motion().x * FACTOR)
            * glam::Quat::from_axis_angle(y.truncate(), response.drag_motion().y * FACTOR);
        self.view = self.view * glam::Mat4::from_quat(rot);
        let mvp = self.mvp_mat();
        // Clone locals so we can move them into the paint callback:
        let rotating_triangle = self.rotating_triangle.clone();
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                rotating_triangle.lock().paint(painter.gl(), mvp);
            })),
        };
        ui.painter().add(callback);
    }
}

struct Scene {
    program: glow::Program,
    vbuf: VertexBuffer<MeshVertex>,
    ibuf: IndexBuffer,
}

impl Scene {
    fn new(gl: &glow::Context) -> Self {
        use glow::HasContext as _;
        let gl: &glow::Context = gl;
        unsafe {
            gl_call!(gl, gl.depth_func(glow::LESS));
            gl_call!(gl, gl.depth_mask(true));
            gl_call!(gl, gl.clear_depth_f32(1.));
            gl_call!(gl, gl.depth_range_f32(0., 1.));
            gl_call!(gl, gl.enable(glow::DEPTH_TEST));
            gl_call!(gl, gl.enable(glow::BLEND));
            gl_call!(
                gl,
                gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA)
            );
            gl_call!(gl, gl.enable(glow::LINE_SMOOTH));
            let program = gl_call!(gl, gl.create_program().expect("Cannot create program"));
            let (vertex_shader_source, fragment_shader_source) = (
                include_str!("shaders/vertex.glsl"),
                include_str!("shaders/fragment.glsl"),
            );
            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];
            let shaders: Vec<_> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl_call!(
                        gl,
                        gl.create_shader(*shader_type)
                            .expect("Cannot create shader")
                    );
                    gl_call!(gl, gl.shader_source(shader, shader_source));
                    gl_call!(gl, gl.compile_shader(shader));
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl_call!(gl, gl.attach_shader(program, shader));
                    shader
                })
                .collect();
            gl_call!(gl, gl.link_program(program));
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );
            for shader in shaders {
                gl_call!(gl, gl.detach_shader(program, shader));
                gl_call!(gl, gl.delete_shader(shader));
            }
            let mut mesh =
                PolyMeshF32::quad_box(glam::Vec3::splat(-0.5), glam::Vec3::splat(0.5)).unwrap();
            mesh.update_face_normals()
                .expect("Cannot update face normals");
            let mut colors = mesh.create_vertex_prop::<glam::Vec3>();
            let mut colors = colors
                .try_borrow_mut()
                .expect("Cannot borrow colors property");
            colors[0] = glam::vec3(0., 0., 0.);
            colors[1] = glam::vec3(1., 0., 0.);
            colors[2] = glam::vec3(1., 1., 0.);
            colors[3] = glam::vec3(0., 1., 0.);
            colors[4] = glam::vec3(0., 0., 1.);
            colors[5] = glam::vec3(1., 0., 1.);
            colors[6] = glam::vec3(1., 1., 1.);
            colors[7] = glam::vec3(0., 1., 1.);
            let vnormals = mesh
                .update_vertex_normals_fast()
                .expect("Cannot update vertex normals");
            let mesh = mesh;
            let vnormals = vnormals.try_borrow().unwrap();
            let vnormals: &[glam::Vec3] = &vnormals;
            let vbuf = VertexBuffer::<MeshVertex>::from_iter(
                mesh.vertices().map(|v| {
                    MeshVertex::new(
                        mesh.point(v).unwrap(),
                        vnormals[v.index() as usize],
                        colors[v.index() as usize],
                    )
                }),
                gl,
            )
            .expect("Failed to create vertex buffer");
            let ibuf = IndexBuffer::from_iter(
                mesh.triangulated_vertices().flatten().map(|v| v.index()),
                gl,
            )
            .expect("Failed to create an index buffer");
            Self {
                program,
                vbuf,
                ibuf,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            self.vbuf.free(gl);
            self.ibuf.free(gl);
        }
    }

    fn paint(&self, gl: &glow::Context, mvp: glam::Mat4) {
        use glow::HasContext as _;
        unsafe {
            gl_call!(gl, gl.clear(glow::DEPTH_BUFFER_BIT));
            gl_call!(gl, gl.use_program(Some(self.program)));
            gl_call!(
                gl,
                gl.uniform_matrix_4_f32_slice(
                    gl.get_uniform_location(self.program, "u_mvp").as_ref(),
                    false,
                    &mvp.to_cols_array(),
                )
            );
            gl_call!(gl, self.vbuf.bind_vao(gl));
            gl_call!(gl, self.ibuf.bind(gl));
            gl_call!(gl, gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL));
            gl_call!(
                gl,
                gl.draw_elements(
                    glow::TRIANGLES,
                    self.ibuf.len() as i32,
                    glow::UNSIGNED_INT,
                    0,
                )
            );
        }
    }
}
