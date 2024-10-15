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

struct PropertyContainer {
    props: Vec<Box<dyn TProperty>>,
}

impl PropertyContainer {
    fn reserve(&mut self, n: usize) {
        for prop in self.props.iter_mut() {
            prop.reserve(n);
        }
    }

    fn resize(&mut self, n: usize) {
        for prop in self.props.iter_mut() {
            prop.resize(n);
        }
    }

    fn clear(&mut self) {
        for prop in self.props.iter_mut() {
            prop.clear();
        }
    }

    fn push(&mut self) {
        for prop in self.props.iter_mut() {
            prop.push();
        }
    }

    fn swap(&mut self, i: usize, j: usize) {
        for prop in self.props.iter_mut() {
            prop.swap(i, j);
        }
    }

    fn copy(&mut self, src: usize, dst: usize) {
        for prop in self.props.iter_mut() {
            prop.copy(src, dst);
        }
    }

    fn len(&self) -> usize {
        let first = match self.props.first() {
            Some(first) => first.len(),
            None => return 0,
        };
        for prop in self.props.iter().skip(1) {
            assert_eq!(first, prop.len());
        }
        return first;
    }
}

trait TPropData: Default + Clone + Copy {}

impl TPropData for glam::Vec3 {}

trait TProperty {
    fn reserve(&mut self, n: usize);

    fn resize(&mut self, n: usize);

    fn clear(&mut self);

    fn push(&mut self);

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

impl<T: TPropData> TProperty for Property<T> {
    fn reserve(&mut self, n: usize) {
        self.data.reserve(n);
    }

    fn resize(&mut self, n: usize) {
        self.data.resize(n, T::default());
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn push(&mut self) {
        self.data.push(T::default());
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
