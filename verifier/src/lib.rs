use nova_software_common as common;

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
pub struct ConfigFile {
    default_state: Option<String>,
    states: Vec<State>,
}

#[derive(Deserialize)]
struct State {
    name: String,
    checks: Vec<Check>,
    commands: Vec<Command>,
    timeout: Option<common::Timeout>,
}

#[derive(Deserialize)]
struct Check {
    name: String,
    check: common::CheckType,
    condition: common::CheckCondition,
    value: f32,
    transition: Option<String>,
}

#[derive(Deserialize)]
struct Command {
    object: String,
    value: f32,
    delay: f32,
}

fn verify_inner(toml: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let input: ConfigFile = toml::from_str(toml)?;
    Err("Error".into())
}

#[pyfunction]
fn verify(toml: String) -> PyResult<Vec<u8>> {
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
