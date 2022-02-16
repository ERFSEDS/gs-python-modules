//! There must be at least one state that doesn't transition to so that we can serialize and
//! deserialize states. This prevents an infinite graph situation

pub mod upper;
pub mod lower;

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::{iter::IterNextOutput, prelude::*, PyIterProtocol};

create_exception!(verifier, TomlParseException, PyException);
create_exception!(verifier, VerifyException, PyException);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("toml parse error {0}")]
    Toml(#[from] toml::de::Error),

    #[error("postcard error {0}")]
    Postcard(#[from] postcard::Error),
}

fn verify_inner(toml: &str) -> Result<Vec<u8>, Error> {
    let mid = upper::verify(toml)?;
    let lower = lower::verify(mid)?;
    Ok(postcard::to_stdvec(&lower)?)
}

#[pyfunction]
fn verify(toml: String) -> PyResult<Vec<u8>> {
    let result = verify_inner(toml.as_str());
    result.map_err(|e| TomlParseException::new_err(format!("{}", e)))
}

#[pymodule]
fn verifier(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("TomlParseException", py.get_type::<TomlParseException>())?;

    m.add("VerifyException", py.get_type::<VerifyException>())?;

    m.add_function(wrap_pyfunction!(verify, m)?)?;

    Ok(())
}
