use postcard::take_from_bytes;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::{iter::IterNextOutput, prelude::*, PyIterProtocol};
use serde::{Deserialize, Serialize};
use std::io::Read;

create_exception!(verifier, TomlParseException, PyException);
create_exception!(verifier, VerifyException, PyException);

#[pyfunction]
fn verify(toml: String) -> PyResult<String> {
    let s =
        toml::from_str(toml.as_str()).map_err(|e| TomlParseException::new_err(format!("{}", e)))?;
    Ok("".to_owned())
}

#[pymodule]
fn verifier(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("TomlParseException", py.get_type::<TomlParseException>())?;

    m.add("VerifyException", py.get_type::<VerifyException>())?;

    m.add_function(wrap_pyfunction!(verify, m)?)?;

    Ok(())
}
