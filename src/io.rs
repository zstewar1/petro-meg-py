use std::io::{self, Read, Seek, Write};

use petro_meg::writer::FileContent;
use pyo3::prelude::*;

pub(crate) trait AnyHelper {
    /// Use this as a bound any.
    fn as_bound<F, R>(&self, f: F) -> R
    where
        F: for<'a, 'py> FnOnce(&'a Bound<'py, PyAny>) -> R;
}

impl<'a, 'py> AnyHelper for &'a Bound<'py, PyAny> {
    fn as_bound<F, R>(&self, f: F) -> R
    where
        F: for<'aa, 'pypy> FnOnce(&'aa Bound<'pypy, PyAny>) -> R,
    {
        f(*self)
    }
}

impl<'py> AnyHelper for Bound<'py, PyAny> {
    fn as_bound<F, R>(&self, f: F) -> R
    where
        F: for<'a, 'pypy> FnOnce(&'a Bound<'pypy, PyAny>) -> R,
    {
        f(self)
    }
}

impl AnyHelper for Py<PyAny> {
    fn as_bound<F, R>(&self, f: F) -> R
    where
        F: for<'a, 'py> FnOnce(&'a Bound<'py, PyAny>) -> R,
    {
        Python::attach(move |py| {
            let bound = self.bind(py);
            f(bound)
        })
    }
}

/// Implements Read on a python object that implements io.RawIOBase or io.BufferedIOBase.
pub(crate) struct AnyAsRead<A> {
    py_read: A,
}

impl<A> AnyAsRead<A> {
    /// Wrap the given py any as a reader.
    pub(crate) fn new(py_read: A) -> Self {
        Self { py_read }
    }

    /// Gets a reference to the inner python object.
    pub(crate) fn inner(&self) -> &A {
        &self.py_read
    }
}

impl<A: AnyHelper> Read for AnyAsRead<A> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.py_read.as_bound(|py_read| {
            let read = py_read.call_method1("read1", (buf.len(),))?;
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
        })
    }
}

impl<A: AnyHelper> Seek for AnyAsRead<A> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.py_read.as_bound(|py_read| {
            let res = match pos {
                io::SeekFrom::Start(offset) => py_read.call_method1("seek", (offset, 0))?,
                io::SeekFrom::Current(offset) => py_read.call_method1("seek", (offset, 1))?,
                io::SeekFrom::End(offset) => py_read.call_method1("seek", (offset, 2))?,
            };
            Ok(res.extract()?)
        })
    }
}

impl<A: AnyHelper> FileContent for AnyAsRead<A> {
    fn ensure_at_start(&mut self) -> io::Result<()> {
        self.rewind()
    }

    fn file_len(&self) -> io::Result<u64> {
        self.py_read.as_bound(|py_read| {
            let old_pos = py_read.call_method0("tell")?.extract::<u64>()?;
            let len = py_read
                .call_method1("seek", (/* offset= */ 0, /* whence=from end */ 2))?
                .extract::<u64>()?;
            py_read.call_method1("seek", (old_pos,))?;
            Ok(len)
        })
    }
}

/// Wraps a python object to make it implement Write.
pub(crate) struct AnyAsWrite<A> {
    py_write: A,
}

impl<A> AnyAsWrite<A> {
    /// Wrap the given PyAny to make it implement Write.
    pub(crate) fn new(py_write: A) -> Self {
        Self { py_write }
    }
}

impl<A: AnyHelper> Write for AnyAsWrite<A> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.py_write.as_bound(|py_write| {
            let res = py_write.call_method1("write", (buf,))?;
            if res.is_none() {
                Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Python write returned None",
                ))
            } else {
                Ok(res.extract::<usize>()?)
            }
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.py_write.as_bound(|py_write| {
            py_write.call_method0("flush")?;
            Ok(())
        })
    }
}
