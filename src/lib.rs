use pyo3::prelude::*;

mod path;
mod reader;

/// Provides support for opening and reading Petroglyph MEGA files in Python.
#[pymodule]
mod petro_meg {
    #[pymodule_export]
    use crate::path::PyMegPath;

    #[pymodule_export]
    use crate::reader::{PyFileEntry, read_meg};
}
