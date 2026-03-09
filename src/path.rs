use std::ops::Deref;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use petro_meg::path::{MegPath, MegPathBuf, MegPathError};
use pyo3::types::{PyString, PyStringMethods};

/// A case-insensitive, ASCII-only relative path used for Petroglyph MEGA files.
#[pyclass(eq, hash, frozen, module = "petro_meg", name = "MegPath")]
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct PyMegPath {
    path: MegPathBuf,
}

#[pymethods]
impl PyMegPath {
    /// Construct a MegPath from a `str`.
    ///
    /// Path must be relative, ASCII only, not end with `/`, use relational components like `.` or
    /// `..`, or any characters that are invalid on Windows path names.
    #[new]
    #[pyo3(signature = (path=None))]
    fn new<'py>(py: Python<'py>, path: Option<&Bound<'py, PyAny>>) -> PyResult<Py<Self>> {
        if let Some(path) = path {
            if let Ok(path) = path.cast::<PyString>() {
                match MegPathBuf::from_string(path.to_cow()?.into_owned()) {
                    Ok(path) => Bound::new(py, PyMegPath { path }).map(|b| b.unbind()),
                    Err(err) => Err(path_value_err(err)),
                }
            } else if let Ok(path) = path.cast::<PyMegPath>() {
                Ok(path.clone().unbind())
            } else {
                Err(PyTypeError::new_err(format!(
                    "Expected str or MegPath, got {}",
                    path.get_type()
                )))
            }
        } else {
            Bound::new(
                py,
                PyMegPath {
                    path: MegPathBuf::new(),
                },
            )
            .map(|b| b.unbind())
        }
    }

    fn __repr__(&self) -> String {
        format!("MegPath({:?})", self.path)
    }

    fn __str__(&self) -> String {
        self.path.to_string()
    }
}

pub(crate) struct BorrowMegPath<'a>(pub &'a MegPath);

impl<'a> Deref for BorrowMegPath<'a> {
    type Target = MegPath;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for BorrowMegPath<'a> {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let path: &'a MegPath = if obj.is_instance_of::<PyString>() {
            let path = <&'a str>::extract(obj)?;
            MegPath::from_str(path).map_err(path_value_err)?
        } else if let Ok(path) = obj.cast::<PyMegPath>() {
            let path: &PyMegPath = path.get();
            // Later versions will have Borrowed::get that preserves the 'a lifetime for us, but on
            // this version we have to include an ugly hack to extend the lifetime.
            let path: &'a PyMegPath = unsafe { std::mem::transmute(path) };
            &path.path
        } else {
            return Err(PyTypeError::new_err(format!(
                "Expected str or MegPath, got {}",
                obj.get_type()
            )));
        };
        Ok(BorrowMegPath(path))
    }
}

/// Maps a MegPathError to a ValueError.
fn path_value_err(err: MegPathError) -> PyErr {
    PyValueError::new_err(err.to_string())
}
