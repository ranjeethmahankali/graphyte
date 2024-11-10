use std::path::PathBuf;

use alum::{element::Handle, PolyMeshF32};
use three_d::{
    degrees, radians, vec3, Camera, ClearState, ColorMaterial, CpuMesh, FrameOutput, Geometry, Gm,
    Indices, Mat4, Mesh, Positions, Srgba, Window, WindowSettings,
};

pub fn main() {
    // Create a window (a canvas on web)
    let window = Window::new(WindowSettings {
        title: "Triangle!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    // Get the graphics context from the window
    let context = window.gl();
    // Create a camera
    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        10.0,
    );

    let mesh = {
        // let mut mesh = alum::PolyMeshF32::quad_box(glam::Vec3::splat(-0.5), glam::Vec3::splat(0.5))
        //     .expect("Cannot create a mesh");
        let mut mesh =
            PolyMeshF32::load_obj(&PathBuf::from("/home/rnjth94/dev/alum/assets/bunny.obj"))
                .expect("Cannot load obj");
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
    // Create a CPU-side mesh consisting of a single colored triangle
    let cpu_mesh = CpuMesh {
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

    // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
    let mut model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

    // Add an animation to the triangle.
    model.set_animation(|time| Mat4::from_angle_y(radians(time * 0.005)));

    // Start the main render loop
    window.render_loop(
        move |frame_input| // Begin a new frame with an updated frame input
    {
        // Ensure the viewport matches the current window viewport which changes if the window is resized
        camera.set_viewport(frame_input.viewport);

        // Update the animation of the triangle
        model.animate(frame_input.accumulated_time as f32);

        // Get the screen render target to be able to render something on the screen
        frame_input.screen()
            // Clear the color and depth of the screen render target
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            // Render the triangle with the color material which uses the per vertex colors defined at construction
            .render(
                &camera, &model, &[]
            );

        // Returns default frame output to end the frame
        FrameOutput::default()
    },
    );
}
