use std::path::PathBuf;

use alum::{element::Handle, PolyMeshF32};
use three_d::{
    degrees, vec3, AmbientLight, Camera, ClearState, CpuMaterial, CpuMesh, Cull, DirectionalLight,
    FrameOutput, Gm, Indices, InnerSpace, InstancedMesh, Instances, Mat4, Mesh, OrbitControl,
    PhysicalMaterial, Positions, Quat, Srgba, Window, WindowSettings,
};

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Wireframe!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let target = vec3(0.0, 2.0, 0.0);
    let scene_radius = 6.0;
    let mut camera = Camera::new_perspective(
        window.viewport(),
        target + scene_radius * vec3(0.6, 0.3, 1.0).normalize(),
        target,
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 0.1 * scene_radius, 100.0 * scene_radius);
    // Create a CPU-side mesh consisting of a single colored triangle
    let (model, etransforms, vtransforms) = {
        let mesh = {
            // let mut mesh = alum::PolyMeshF32::quad_box(glam::Vec3::splat(-0.5), glam::Vec3::splat(0.5))
            //     .expect("Cannot create a mesh");
            let mut mesh =
                PolyMeshF32::load_obj(&PathBuf::from("/home/rnjth94/dev/alum/assets/bunny.obj"))
                    .expect("Cannot load obj");
            {
                let mut points = mesh.points();
                let mut points = points.try_borrow_mut().expect("Cannot borrow points");
                for p in points.iter_mut() {
                    *p = *p * 10.; // Scale the mesh.
                }
            }
            mesh.update_face_normals()
                .expect("Cannot update face normals");
            mesh.update_vertex_normals_fast()
                .expect("Cannot update vertex normals");
            mesh
        };
        let points = mesh.points();
        let points = points.try_borrow().expect("Cannot borrow points");
        let vnormals = mesh.vertex_normals().expect("Cannot borrow vertex normals");
        let vnormals = vnormals.try_borrow().expect("Cannot borrow vertex normals");
        let cpumesh = CpuMesh {
            positions: Positions::F32(points.iter().map(|p| 2. * vec3(p.x, p.y, p.z)).collect()),
            indices: Indices::U32(
                mesh.triangulated_vertices()
                    .flatten()
                    .map(|v| v.index())
                    .collect(),
            ),
            colors: Some(vec![Srgba::new(128, 128, 128, 255); points.len()]),
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
                        let h = mesh.edge_halfedge(e, false);
                        let mut ev = mesh.calc_halfedge_vector(h, &points);
                        let length = ev.length();
                        ev /= length;
                        let start = points[mesh.from_vertex(h).index() as usize];
                        let start = vec3(start.x, start.y, start.z);
                        Mat4::from_translation(start)
                            * Into::<Mat4>::into(Quat::from_arc(
                                vec3(1.0, 0., 0.0),
                                vec3(ev.x, ev.y, ev.z),
                                None,
                            ))
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
    sphere.transform(&Mat4::from_scale(0.01)).unwrap();
    let vertices = Gm::new(
        InstancedMesh::new(&context, &vtransforms, &sphere),
        wireframe_material.clone(),
    );
    let mut cylinder = CpuMesh::cylinder(10);
    cylinder
        .transform(&Mat4::from_nonuniform_scale(1.0, 0.007, 0.007))
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
