use petro_meg::version::MegVersion;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

/// Extract type for reader version arg.
pub(crate) struct VersionArg {
    pub version: MegVersion,
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
