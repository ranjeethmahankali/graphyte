use std::sync::{Arc, RwLock};

enum Error {
    SyncFailed,
}

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
    fn reserve(&mut self, n: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.reserve(n)?;
        }
        return Ok(());
    }

    fn resize(&mut self, n: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.resize(n)?;
        }
        return Ok(());
    }

    fn clear(&mut self) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.clear()?;
        }
        return Ok(());
    }

    fn push(&mut self) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.push()?;
        }
        return Ok(());
    }

    fn swap(&mut self, i: usize, j: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.swap(i, j)?;
        }
        return Ok(());
    }

    fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.copy(src, dst)?;
        }
        return Ok(());
    }

    fn len(&self) -> Result<usize, Error> {
        let first = match self.props.first() {
            Some(first) => first.len()?,
            None => return Ok(0),
        };
        for prop in self.props.iter().skip(1) {
            assert_eq!(first, prop.len()?);
        }
        return Ok(first);
    }
}

trait TPropData: Default + Clone + Copy {}

impl TPropData for glam::Vec3 {}

trait TProperty {
    fn reserve(&mut self, n: usize) -> Result<(), Error>;

    fn resize(&mut self, n: usize) -> Result<(), Error>;

    fn clear(&mut self) -> Result<(), Error>;

    fn push(&mut self) -> Result<(), Error>;

    fn swap(&mut self, i: usize, j: usize) -> Result<(), Error>;

    fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error>;

    fn len(&self) -> Result<usize, Error>;
}

struct Property<T: TPropData> {
    data: Arc<RwLock<Vec<T>>>,
}

impl<T: TPropData> Default for Property<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<T: TPropData> TProperty for Property<T> {
    fn reserve(&mut self, n: usize) -> Result<(), Error> {
        self.data.write().map_err(|_| Error::SyncFailed)?.reserve(n); // reserve memory.
        return Ok(());
    }

    fn resize(&mut self, n: usize) -> Result<(), Error> {
        self.data
            .write()
            .map_err(|_| Error::SyncFailed)?
            .resize(n, T::default());
        return Ok(());
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.data.write().map_err(|_| Error::SyncFailed)?.clear();
        return Ok(());
    }

    fn push(&mut self) -> Result<(), Error> {
        self.data
            .write()
            .map_err(|_| Error::SyncFailed)?
            .push(T::default());
        return Ok(());
    }

    fn swap(&mut self, i: usize, j: usize) -> Result<(), Error> {
        self.data.write().map_err(|_| Error::SyncFailed)?.swap(i, j);
        return Ok(());
    }

    fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error> {
        self.data
            .write()
            .map_err(|_| Error::SyncFailed)?
            .copy_within(src..(src + 1), dst);
        return Ok(());
    }

    fn len(&self) -> Result<usize, Error> {
        Ok(self.data.read().map_err(|_| Error::SyncFailed)?.len())
    }
}
