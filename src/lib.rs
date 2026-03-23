use pyo3::prelude::*;

mod io;
mod path;
mod reader;
mod version;
mod writer;

/// Provides support for opening and reading Petroglyph MEGA files in Python.
#[pymodule]
mod petro_meg {
    #[pymodule_export]
    use crate::path::PyMegPath;

    #[pymodule_export]
    use crate::reader::{PyFileEntry, read_meg};

    #[pymodule_export]
    use crate::writer::PyMegBuilder;
}
