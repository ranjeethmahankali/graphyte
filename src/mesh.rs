struct Vertex {
    halfedge: Option<u32>,
}

struct Halfedge {
    face: Option<u32>,
    vertex: u32,
    next: u32,
    prev: u32,
}

struct Edge {
    halfedges: [Halfedge; 2],
}

struct Face {
    halfedge: u32,
}

struct Mesh {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
    points: Property<glam::Vec3>,
}

trait TPropData: Default + Clone + Copy {}

impl TPropData for glam::Vec3 {}

trait TProperty<T: TPropData> {
    fn reserve(&mut self, n: usize);

    fn resize(&mut self, n: usize);

    fn clear(&mut self);

    fn push(&mut self, val: T);

    fn swap(&mut self, i: usize, j: usize);

    fn copy(&mut self, src: usize, dst: usize);

    fn len(&self) -> usize;
}

struct Property<T: TPropData> {
    data: Vec<T>,
}

impl<T: TPropData> Default for Property<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<T: TPropData> TProperty<T> for Property<T> {
    fn reserve(&mut self, n: usize) {
        self.data.reserve(n);
    }

    fn resize(&mut self, n: usize) {
        self.data.resize(n, T::default());
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn push(&mut self, val: T) {
        self.data.push(val);
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.data.swap(i, j);
    }

    fn copy(&mut self, src: usize, dst: usize) {
        self.data.copy_within(src..(src + 1), dst);
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}
