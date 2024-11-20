use alum::{
    Adaptor, CrossProductAdaptor, Decimater, DotProductAdaptor, FloatScalarAdaptor, Handle,
    HasIterators, PolyMeshT, QuadricDecimater, VectorAngleAdaptor, VectorLengthAdaptor,
    VectorNormalizeAdaptor,
};
use three_d::{InnerSpace, Vec3};

pub struct MeshAdaptor {}

impl Adaptor<3> for MeshAdaptor {
    type Vector = Vec3;

    type Scalar = f32;

    fn vector(coords: [Self::Scalar; 3]) -> Self::Vector {
        three_d::vec3(coords[0], coords[1], coords[2])
    }

    fn zero_vector() -> Self::Vector {
        three_d::vec3(0.0, 0.0, 0.0)
    }

    fn vector_coord(v: &Self::Vector, i: usize) -> Self::Scalar {
        v[i]
    }
}

impl FloatScalarAdaptor<3> for MeshAdaptor {
    fn scalarf32(val: f32) -> Self::Scalar {
        val
    }

    fn scalarf64(val: f64) -> Self::Scalar {
        val as f32
    }
}

impl VectorNormalizeAdaptor<3> for MeshAdaptor {
    fn normalized_vec(v: Self::Vector) -> Self::Vector {
        v.normalize()
    }
}

impl CrossProductAdaptor for MeshAdaptor {
    fn cross_product(a: Self::Vector, b: Self::Vector) -> Self::Vector {
        a.cross(b)
    }
}

impl DotProductAdaptor<3> for MeshAdaptor {
    fn dot_product(a: Self::Vector, b: Self::Vector) -> Self::Scalar {
        a.dot(b)
    }
}

impl VectorLengthAdaptor<3> for MeshAdaptor {
    fn vector_length(v: Self::Vector) -> Self::Scalar {
        v.magnitude()
    }
}

impl VectorAngleAdaptor for MeshAdaptor {
    fn vector_angle(a: Self::Vector, b: Self::Vector) -> Self::Scalar {
        a.angle(b).0
    }
}

pub type PolyMesh = PolyMeshT<3, MeshAdaptor>;

pub struct ExperimentDecimater {
    inner: QuadricDecimater<MeshAdaptor>,
    history: Vec<PolyMesh>,
    mid_point: Vec3,
}

impl ExperimentDecimater {
    pub fn new(mesh: &PolyMesh) -> Self {
        ExperimentDecimater {
            inner: QuadricDecimater::new(mesh).expect("Cannot create quadric decimater"),
            history: Vec::new(),
            mid_point: Vec3::unit_x(),
        }
    }

    pub fn history(&self) -> &[PolyMesh] {
        &self.history
    }
}

impl Decimater<PolyMesh> for ExperimentDecimater {
    type Cost = f32;

    fn collapse_cost(&self, mesh: &PolyMesh, h: alum::HH) -> Option<Self::Cost> {
        let cost = self.inner.collapse_cost(mesh, h);
        if let Some(cost) = cost {}
        cost
    }

    fn before_collapse(&mut self, mesh: &PolyMesh, h: alum::HH) -> Result<(), alum::Error> {
        self.inner.before_collapse(mesh, h)?;
        let points = mesh.points();
        let points = points.try_borrow()?;
        self.mid_point =
            (points[h.tail(mesh).index() as usize] + points[h.head(mesh).index() as usize]) * 0.5;
        Ok(())
    }

    fn after_collapse(&mut self, mesh: &PolyMesh, v: alum::VH) -> Result<(), alum::Error> {
        self.inner.after_collapse(mesh, v)?;
        if let Some(mut fnormals) = mesh.face_normals() {
            let mut fnormals = fnormals.try_borrow_mut()?;
            let mut points = mesh.points();
            let mut points = points.try_borrow_mut()?;
            points[v.index() as usize] = self.mid_point;
            for f in mesh.vf_ccw_iter(v) {
                fnormals[f.index() as usize] = mesh.calc_face_normal(f, &points);
            }
            if let Some(mut vnormals) = mesh.vertex_normals() {
                let mut vnormals = vnormals.try_borrow_mut()?;
                vnormals[v.index() as usize] = mesh.calc_vertex_normal_fast(v, &fnormals);
            }
        }
        let mut copy = mesh.clone();
        copy.garbage_collection()?;
        self.history.push(copy);
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use alum::{HasIterators, HasTopology};

    use super::PolyMesh;

    macro_rules! assert_float_eq {
        ($a:expr, $b:expr, $eps:expr, $debug:expr) => {{
            // Make variables to avoid evaluating experssions multiple times.
            let a = $a;
            let b = $b;
            let eps = $eps;
            let error = (a - b).abs();
            if error > eps {
                eprintln!("{:?}", $debug);
            }
            assert!(
                error <= eps,
                "Assertion failed: |({}) - ({})| = {:e} <= {:e}",
                a,
                b,
                error,
                eps
            );
        }};
        ($a:expr, $b:expr, $eps:expr) => {
            assert_float_eq!($a, $b, $eps, "")
        };
        ($type:ty, $a:expr, $b:expr) => {
            assert_float_eq!($type, $a, $b, $type::EPSILON)
        };
    }

    macro_rules! assert_f32_eq {
        ($a:expr, $b:expr, $eps:expr, $debug:expr) => {
            assert_float_eq!($a, $b, $eps, $debug)
        };
        ($a:expr, $b:expr, $eps:expr) => {
            assert_float_eq!($a, $b, $eps)
        };
        ($a:expr, $b:expr) => {
            assert_float_eq!($a, $b, f32::EPSILON)
        };
    }

    #[test]
    fn t_quad_box() {
        let qbox = PolyMesh::quad_box(three_d::vec3(0., 0., 0.), three_d::vec3(1., 1., 1.))
            .expect("Cannot create a quad box mesh");
        assert_eq!(qbox.num_vertices(), 8);
        assert_eq!(qbox.num_halfedges(), 24);
        assert_eq!(qbox.num_edges(), 12);
        assert_eq!(qbox.num_faces(), 6);
        for v in qbox.vertices() {
            assert_eq!(qbox.vf_ccw_iter(v).count(), 3);
        }
        assert_eq!(1., qbox.try_calc_volume().expect("Cannot compute volume"));
        assert_eq!(6., qbox.try_calc_area().expect("Cannot compute area"));
    }

    #[test]
    fn t_tetrahedron() {
        let tet = PolyMesh::tetrahedron(1.0).expect("Cannot create a tetrahedron");
        assert_eq!(4, tet.num_vertices());
        assert_eq!(12, tet.num_halfedges());
        assert_eq!(6, tet.num_edges());
        assert_eq!(4, tet.num_faces());
        assert_eq!(
            8.0 / 3.0f32.sqrt(),
            tet.try_calc_area().expect("Cannot compute area")
        );
        assert_f32_eq!(
            8.0 / (9.0 * 3.0f32.sqrt()),
            tet.try_calc_volume().expect("Cannot compute volume")
        );
    }

    #[test]
    fn t_hexahedron() {
        let hex = PolyMesh::hexahedron(1.0).expect("Cannot create hexahedron");
        assert_eq!(hex.num_vertices(), 8);
        assert_eq!(hex.num_halfedges(), 24);
        assert_eq!(hex.num_edges(), 12);
        assert_eq!(hex.num_faces(), 6);
        assert_f32_eq!(8.0, hex.try_calc_area().expect("Cannot compute area"), 1e-6);
        assert_f32_eq!(
            8.0 / (3.0 * 3.0f32.sqrt()),
            hex.try_calc_volume().expect("Cannot compute volume")
        );
    }

    #[test]
    fn t_octahedron() {
        let oct = PolyMesh::octahedron(1.0).expect("Cannot create octahedron");
        assert_eq!(oct.num_vertices(), 6);
        assert_eq!(oct.num_halfedges(), 24);
        assert_eq!(oct.num_edges(), 12);
        assert_eq!(oct.num_faces(), 8);
        assert_eq!(
            4.0 * 3.0f32.sqrt(),
            oct.try_calc_area().expect("Cannot compute area")
        );
        assert_f32_eq!(
            4.0 / 3.0,
            oct.try_calc_volume().expect("Cannot compute volume")
        );
    }

    #[test]
    fn t_icosahedron() {
        let ico = PolyMesh::icosahedron(1.0).expect("Cannot create icosahedron");
        assert_eq!(12, ico.num_vertices());
        assert_eq!(60, ico.num_halfedges());
        assert_eq!(30, ico.num_edges());
        assert_eq!(20, ico.num_faces());
        assert_f32_eq!(
            {
                let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
                20.0 * 3.0f32.sqrt() / (phi * phi + 1.0)
            },
            ico.try_calc_area().expect("Cannot compute area"),
            1e-6
        );
        assert_f32_eq!(
            {
                let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
                20.0 * phi * phi / (3.0 * (phi * phi + 1.0) * (phi * phi + 1.0).sqrt())
            },
            ico.try_calc_volume().expect("Cannot compute volume"),
            1e-6
        );
    }

    #[test]
    fn t_dodecahedron() {
        let dod = PolyMesh::dodecahedron(1.0).expect("Cannot create dodecahedron");
        assert_eq!(20, dod.num_vertices());
        assert_eq!(60, dod.num_halfedges());
        assert_eq!(30, dod.num_edges());
        assert_eq!(12, dod.num_faces());
        assert_f32_eq!(
            {
                let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
                20.0f32 / (phi * (3.0f32 - phi).sqrt())
            },
            dod.try_calc_area().expect("Cannot compute area"),
            1e-6
        );
        assert_f32_eq!(
            {
                let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
                40.0 / (3.0 * 3.0f32.sqrt() * (6.0 - 2.0 * phi))
            },
            dod.try_calc_volume().expect("Cannot compute volume")
        );
    }
}
