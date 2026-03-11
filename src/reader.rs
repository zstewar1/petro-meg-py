use std::io::{self, Read, Seek};
use std::sync::{Arc, OnceLock};

use petro_meg::crypto::Key;
use petro_meg::reader::{FileEntry, MegReadError, MegReadOptions, ReadMegMeta};
use petro_meg::version::MegVersion;
use pyo3::exceptions::{PyIOError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::sync::OnceLockExt;
use pyo3::{PyTraverseError, PyVisit};

use crate::path::PyMegPath;

/// Gets a list of files from the given MEGA file.
#[pyfunction]
#[pyo3(signature = (mega_file, /, version=None, key=None, iv=None))]
pub(crate) fn read_meg(
    mega_file: Bound<PyAny>,
    version: Option<VersionArg>,
    key: Option<&[u8]>,
    iv: Option<&[u8]>,
) -> PyResult<Vec<PyFileEntry>> {
    let version = version.map(|v| v.version);
    let mut read = AnyAsRead {
        py_read: &mega_file,
    };
    let key = match (key, iv) {
        (Some(k), Some(i)) if k.len() == 16 && k.len() == 16 => {
            let mut key = [0u8; 16];
            key.copy_from_slice(k);
            let mut iv = [0u8; 16];
            iv.copy_from_slice(i);
            Some(Key::new(key, iv))
        }
        (Some(_), Some(_)) => {
            return Err(PyValueError::new_err("Key and IV must both have len 16"));
        }
        (None, Some(_)) | (Some(_), None) => {
            return Err(PyTypeError::new_err(
                "Key and IV must either both be specified or both None",
            ));
        }
        (None, None) => None,
    };
    let mut options = MegReadOptions::new();
    options.set_key(key);
    let options = Arc::new(options);

    let files = version
        .read_meg_meta_opt(&mut read, &options)
        .map_err(|e| match e {
            MegReadError::IoError(e) => PyErr::from(e),
            _ => PyValueError::new_err(format!("{e}")),
        })?
        .into_iter()
        .map(|entry| PyFileEntry {
            entry,
            name: OnceLock::new(),
            mega_file: Some(mega_file.clone().unbind()),
            options: options.clone(),
        })
        .collect();
    Ok(files)
}

/// Wrapper for FileEntry used by Python.
#[pyclass(module = "petro_meg", name = "FileEntry")]
pub(crate) struct PyFileEntry {
    entry: FileEntry,
    name: OnceLock<PyResult<Py<PyMegPath>>>,
    mega_file: Option<Py<PyAny>>,
    options: Arc<MegReadOptions>,
}

#[pymethods]
impl PyFileEntry {
    /// Gets the name of this file as a MegPath.
    #[getter]
    fn name(&self, py: Python) -> PyResult<Py<PyMegPath>> {
        let name = self.name.get_or_init_py_attached(py, || {
            let path: PyMegPath = self.entry.name().to_owned().into();
            Py::new(py, path)
        });
        match name {
            Ok(path) => Ok(path.clone_ref(py)),
            Err(e) => Err(e.clone_ref(py)),
        }
    }

    /// Gets the size of this file.
    #[getter]
    fn size(&self) -> u32 {
        self.entry.size()
    }

    /// Reads the contents of the file. The original MEGA file must still be open.
    fn read(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let Some(mega_file) = self.mega_file.as_ref() else {
            return Err(PyIOError::new_err("Already disposed"));
        };
        let py_read = mega_file.bind(py);
        let reader = AnyAsRead { py_read };
        let mut reader = self.entry.extract_from(reader, &self.options)?;

        let mut res = Vec::with_capacity(self.entry.size() as usize);
        reader.read_to_end(&mut res)?;
        Ok(res)
    }

    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        visit.call(&self.mega_file)?;
        Ok(())
    }

    fn __clear__(&mut self) {
        self.mega_file = None;
    }
}

/// Implements Read on a python object that implements io.RawIOBase or io.BufferedIOBase.
struct AnyAsRead<'a, 'py> {
    py_read: &'a Bound<'py, PyAny>,
}

impl<'a, 'py> Read for AnyAsRead<'a, 'py> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.py_read.call_method1("read1", (buf.len(),))?;
        let data = read.extract::<&[u8]>().map_err(|e| PyErr::from(e))?;
        if data.len() > buf.len() {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Underlying python File returned more bytes than requested",
            ))
        } else {
            buf[..data.len()].copy_from_slice(data);
            Ok(data.len())
        }
    }
}

impl<'a, 'py> Seek for AnyAsRead<'a, 'py> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let res = match pos {
            io::SeekFrom::Start(offset) => self.py_read.call_method("seek", (offset, 0), None)?,
            io::SeekFrom::Current(offset) => self.py_read.call_method("seek", (offset, 1), None)?,
            io::SeekFrom::End(offset) => self.py_read.call_method("seek", (offset, 2), None)?,
        };
        Ok(res.extract()?)
    }
}

/// Extract type for reader version arg.
pub(crate) struct VersionArg {
    version: MegVersion,
}

impl<'a, 'py> FromPyObject<'a, 'py> for VersionArg {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let version = if let Ok(v) = obj.extract::<u64>() {
            match v {
                1 => MegVersion::V1,
                2 => MegVersion::V2,
                3 => MegVersion::V3,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Version must be 1, 2, or 3, got {v}"
                    )));
                }
            }
        } else if let Ok(v) = obj.extract::<&str>() {
            match v {
                "v1" | "V1" | "1" => MegVersion::V1,
                "v2" | "V2" | "2" => MegVersion::V2,
                "v3" | "V3" | "3" => MegVersion::V3,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Version must be 1, 2, or 3, got {v}"
                    )));
                }
            }
        } else {
            return Err(PyTypeError::new_err(format!(
                "Version must be int, or str, got {}",
                obj.get_type(),
            )));
        };
        Ok(VersionArg { version })
    }
}
