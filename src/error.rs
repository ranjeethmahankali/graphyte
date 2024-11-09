use std::fmt::Debug;

pub enum Error {
    GLError(String),
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GLError(message) => write!(f, "OpenGL Error: {}", message),
        }
    }
}
