//! Wraps [`nova-verifier`] into a native python library so that the ground station can verify
//! config files easily from python

create_exception!(verifier, TomlParseException, PyException);
create_exception!(verifier, VerifyException, PyException);

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

#[pyfunction]
fn verify(toml: String) -> PyResult<Vec<u8>> {
    let result = nova_verifier::verify(toml.as_str());
    result.map_err(|e| TomlParseException::new_err(format!("{}", e)))
}

#[pymodule]
fn nova_verifier(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("TomlParseException", py.get_type::<TomlParseException>())?;

    m.add("VerifyException", py.get_type::<VerifyException>())?;

    m.add_function(wrap_pyfunction!(verify, m)?)?;

    Ok(())
}
