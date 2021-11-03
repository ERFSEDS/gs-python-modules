use nova_software_common::ConfigFile;
use postcard::take_from_bytes;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::{iter::IterNextOutput, prelude::*, PyIterProtocol};
use serde::{Deserialize, Serialize};
use std::error::Error;

create_exception!(verifier, TomlParseException, PyException);
create_exception!(verifier, VerifyException, PyException);

#[derive(Deserialize)]
struct TomlRoot {
    states: Vec<State>,
}

#[derive(Deserialize)]
struct State {
    name: String,
    transition: Vec<String>,
    checks: Vec<Check>,
    commands: Vec<Command>,
}

#[derive(Deserialize)]
struct Check {
    condition: String,
    name: String,
    #[serde(rename = "type")]
    check_type: String,
    value: f32,
}

#[derive(Deserialize)]
struct Command {
    object: String,
    time: f32,
    value: f32,
}

fn verify_inner(toml: &str) -> Result<String, Box<dyn Error>> {
    let input: TomlRoot = toml::from_str(toml)?;
    Err("Error".into())
}

#[pyfunction]
fn verify(toml: String) -> PyResult<String> {
    let result = verify_inner(toml.as_str());
    Ok(result.map_err(|e| TomlParseException::new_err(format!("{}", e)))?)
}

#[pymodule]
fn verifier(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("TomlParseException", py.get_type::<TomlParseException>())?;

    m.add("VerifyException", py.get_type::<VerifyException>())?;

    m.add_function(wrap_pyfunction!(verify, m)?)?;

    Ok(())
}
