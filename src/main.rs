mod mesh;
mod scene;

use alum::{Handle, HasIterators, HasTopology};
use mesh::PolyMesh;
use scene::CameraMouseControl;
use std::{path::PathBuf, time::Instant};
use three_d::{
    degrees, vec3, AmbientLight, Camera, ClearState, CpuMaterial, CpuMesh, Cull, DirectionalLight,
    FrameOutput, Gm, Indices, InnerSpace, InstancedMesh, Instances, Mat4, Mesh, PhysicalMaterial,
    Positions, Quat, Srgba, Window, WindowSettings,
};

fn bunny_mesh() -> PolyMesh {
    let mesh = PolyMesh::load_obj(&PathBuf::from("/home/rnjth94/dev/alum/assets/bunny.obj"))
        .expect("Cannot load obj");
    {
        let mut points = mesh.points();
        let mut points = points.try_borrow_mut().expect("Cannot borrow points");
        for p in points.iter_mut() {
            *p = *p * 10.; // Scale the mesh.
        }
    }
    mesh
}

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Viewer".to_string(),
        min_size: (512, 256),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();
    let mesh = {
        // let mut mesh = PolyMesh::icosahedron(1.0).expect("Cannoto create icosahedron");
        // let mut mesh = bunny_mesh();
        let mut mesh = PolyMesh::unit_box().expect("Cannot make a box");
        let before = Instant::now();
        mesh.subdivide_loop(1, false).expect("Cannot do subd");
        let duration = Instant::now() - before;
        println!("Subdivision took {}ms", duration.as_millis());
        println!(
            "This mesh has {} boundary vertices.",
            mesh.vertices().filter(|v| v.is_boundary(&mesh)).count()
        );
        mesh.update_face_normals()
            .expect("Cannot update face normals");
        mesh.update_vertex_normals_fast()
            .expect("Cannot update vertex normals");
        mesh
    };
    let target = vec3(0.0, 1.0, 0.0);
    let scene_radius: f32 = 6.0;
    let mut camera = Camera::new_perspective(
        window.viewport(),
        target + scene_radius * vec3(0.6, 0.3, 1.0).normalize(),
        target,
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control =
        CameraMouseControl::new(*camera.target(), 0.1 * scene_radius, 100.0 * scene_radius);
    // Create a CPU-side mesh consisting of a single colored triangle
    let (model, etransforms, vtransforms) = {
        let points = mesh.points();
        let points = points.try_borrow().expect("Cannot borrow points");
        let vnormals = mesh.vertex_normals().expect("Cannot borrow vertex normals");
        let vnormals = vnormals.try_borrow().expect("Cannot borrow vertex normals");
        let cpumesh = CpuMesh {
            positions: Positions::F32(points.iter().map(|p| vec3(p.x, p.y, p.z)).collect()),
            indices: Indices::U32(
                mesh.triangulated_vertices()
                    .flatten()
                    .map(|v| v.index())
                    .collect(),
            ),
            normals: Some(vnormals.iter().map(|n| vec3(n.x, n.y, n.z)).collect()),
            ..Default::default()
        };
        let model_material = PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::new_opaque(200, 200, 200),
                roughness: 0.7,
                metallic: 0.8,
                ..Default::default()
            },
        );
        (
            Gm::new(Mesh::new(&context, &cpumesh), model_material),
            Instances {
                transformations: mesh
                    .edges()
                    .map(|e| {
                        let h = e.halfedge(false);
                        let mut ev = mesh.calc_halfedge_vector(h, &points);
                        let length = ev.magnitude();
                        ev /= length;
                        let ev = vec3(ev.x, ev.y, ev.z);
                        let start = points[h.tail(&mesh).index() as usize];
                        let start = vec3(start.x, start.y, start.z);
                        Mat4::from_translation(start)
                            * Into::<Mat4>::into(Quat::from_arc(vec3(1.0, 0., 0.0), ev, None))
                            * Mat4::from_nonuniform_scale(length, 1., 1.)
                    })
                    .collect(),
                ..Default::default()
            },
            Instances {
                transformations: points
                    .iter()
                    .map(|pos| Mat4::from_translation(vec3(pos.x, pos.y, pos.z)))
                    .collect(),
                ..Default::default()
            },
        )
    };
    let mut wireframe_material = PhysicalMaterial::new_opaque(
        &context,
        &CpuMaterial {
            albedo: Srgba::new_opaque(220, 50, 50),
            roughness: 0.7,
            metallic: 0.8,
            ..Default::default()
        },
    );
    wireframe_material.render_states.cull = Cull::Back;
    let mut sphere = CpuMesh::sphere(8);
    sphere.transform(&Mat4::from_scale(0.001)).unwrap();
    let vertices = Gm::new(
        InstancedMesh::new(&context, &vtransforms, &sphere),
        wireframe_material.clone(),
    );
    let mut cylinder = CpuMesh::cylinder(10);
    cylinder
        .transform(&Mat4::from_nonuniform_scale(1.0, 0.0005, 0.0005))
        .unwrap();
    let edges = Gm::new(
        InstancedMesh::new(&context, &etransforms, &cylinder),
        wireframe_material,
    );
    let ambient = AmbientLight::new(&context, 0.7, Srgba::WHITE);
    let directional0 = DirectionalLight::new(&context, 2.0, Srgba::WHITE, &vec3(-1.0, -1.0, -1.0));
    let directional1 = DirectionalLight::new(&context, 2.0, Srgba::WHITE, &vec3(1.0, 1.0, 1.0));
    // render loop
    window.render_loop(move |mut frame_input| {
        let mut redraw = frame_input.first_frame;
        redraw |= camera.set_viewport(frame_input.viewport);
        redraw |= control.handle_events(&mut camera, &mut frame_input.events);
        if redraw {
            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.1, 0.1, 0.1, 1.0, 1.0))
                .render(
                    &camera,
                    model.into_iter().chain(&vertices).chain(&edges),
                    &[&ambient, &directional0, &directional1],
                );
        }
        FrameOutput {
            swap_buffers: redraw,
            ..Default::default()
        }
    });
}
