use pyo3::prelude::*;

mod path;

/// Provides support for opening and reading Petroglyph MEGA files in Python.
#[pymodule]
mod petro_meg {
    use pyo3::prelude::*;

    #[pymodule_export]
    use crate::path::PyMegPath;
}
