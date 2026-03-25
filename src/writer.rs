use petro_meg::crypto::Key;
use petro_meg::writer::{AnyVersionSettings, BuildMeg, MegBuilder};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::{prelude::*, PyTraverseError, PyVisit};

use crate::io::{AnyAsRead, AnyAsWrite};
use crate::path::BorrowMegPath;
use crate::version::VersionArg;

/// Builder for creating MEGA files.
#[pyclass(mapping, module = "petro_meg", name = "_MegBuilder")]
pub(crate) struct PyMegBuilder {
    inner: MegBuilder<AnyAsRead<Py<PyAny>>, AnyVersionSettings>,
}

#[pymethods]
impl PyMegBuilder {
    /// Creates a MegBuilder for the given version.
    #[new]
    fn new(version: VersionArg) -> Self {
        Self {
            inner: version.version.builder(),
        }
    }

    /// Insert an entry into the builder.
    fn insert(&mut self, path: BorrowMegPath, file: Py<PyAny>) {
        self.inner.insert(path.to_owned(), AnyAsRead::new(file));
    }

    /// Enables encryption with the given key and initial vector.
    fn set_encryption(&mut self, k: &[u8], i: &[u8]) -> PyResult<()> {
        if k.len() == 16 && i.len() == 16 {
            let mut key = [0u8; 16];
            key.copy_from_slice(k);
            let mut iv = [0u8; 16];
            iv.copy_from_slice(i);
            self.inner.set_encryption(Some(Key::new(key, iv)));
            Ok(())
        } else {
            return Err(PyValueError::new_err("Key and IV must both have len 16"));
        }
    }

    /// Build the MEGA file, writing output to the given file/writer. This will clear the builder.
    fn build<'py>(&mut self, out: Bound<'py, PyAny>) -> PyResult<()> {
        let mut new = self.inner.version().builder();
        new.set_encryption(self.inner.encryption().cloned());
        let inner = std::mem::replace(&mut self.inner, new);
        let mut out = AnyAsWrite::new(out);

        inner
            .build(&mut out)
            .map_err(|e| PyIOError::new_err(format!("Failed to build MEGA file: {e}")))?;
        Ok(())
    }

    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        self.inner.files().map(|val| visit.call(val.inner())).collect()
    }

    fn __clear__(&mut self) {
        let new = self.inner.version().builder();
        self.inner = new;
    }
}
