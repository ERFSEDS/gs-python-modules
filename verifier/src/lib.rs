//! There must be at least one state that doesn't transition to so that we can serialize and
//! deserialize states. This prevents an infinite graph situation

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

#[derive(Deserialize, Serialize)]
pub struct ConfigFile {
    default_state: Option<String>,
    states: Vec<State>,
}


#[derive(Deserialize, Serialize)]
struct Timeout {
    /// How long this state can execute in seconds before the rocket automatically transitions to
    /// `state`
    seconds: f32,

    /// The state to transition to when `state`
    state: String,
}

#[derive(Deserialize, Serialize)]
struct State {
    /// The name of this state
    name: String,

    /// How long this state can execute until it automatically transitions to another
    timeout: Option<Timeout>,

    checks: Vec<Check>,
    //commands: Vec<Command>,
}

/// Something relating to the external environment that the rocket will check to determine a future
/// course of action. Examples include:
/// - Transitioning from the `Ground` state to the `Launched` state if altitude is past a certain
/// threshold
/// - Aborting the flight if there is no continuity on the pyro channels
#[derive(Deserialize, Serialize)]
struct Check {
    /// The name describing this check
    name: String,

    /// The name of the thing to be checked
    /// Currently only the strings `altitude`, `pyro1`, `pyro2`, and `pyro3` are supported, and
    /// enable specific filtering conditions
    check: String,

    /// If set, this check will execute when the value of `self.check` > the inner value
    /// Only available for `altitude` checks
    greater_than: Option<f32>,

    /// Forms a check range with `lower_bound` that checks if `check` is in a particular range
    /// Only available for `altitude` checks
    upper_bound: Option<f32>,

    /// Checks if a boolean flag is set or unset
    /// The pyro values are supported
    /// `flag = "set"` or `flag = "unset"`
    flag: Option<String>,

    /// Must be Some(...) if `upper_bound` is Some(...), and must be None if `upper_bound` is none
    lower_bound: Option<f32>,

    /// The name of the state to transition to when when the check is tripped
    transition: Option<String>,
}

#[derive(Deserialize, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Used for format compatibility guarantees. Call with real encoded config files once we have
    /// a stable version to maintain
    fn assert_config_eq(bytes: Vec<u8>, config: common::ConfigFile) {
        let decoded: common::ConfigFile = postcard::from_bytes(bytes.as_slice()).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn basic_serialize1() {
        let config = ConfigFile {
            default_state: Some("PowerOn".to_owned()),
            states: vec![State {
                name: "PowerOn".to_owned(),
                checks: vec![],
                //commands: vec![],
                timeout: None,
            }],
        };
        let expected = r#"default_state = "PowerOn"

[[states]]
name = "PowerOn"
checks = []
"#;

        let real = toml::to_string(&config).unwrap();
        assert_eq!(real.as_str(), expected);
    }

    #[test]
    fn basic_serialize2() {
        let config = ConfigFile {
            default_state: Some("PowerOn".to_owned()),
            states: vec![State {
                name: "PowerOn".to_owned(),
                timeout: None,
                checks: vec![Check {
                    name: "Takeoff".to_owned(),
                    check: "altitude".to_owned(),
                    greater_than: Some(100.0),
                    transition: None,
                }],
                //checks: vec![],
                //commands: vec![],
            }],
        };
        let expected = r#"default_state = "PowerOn"

[[states]]
name = "PowerOn"

[[states.checks]]
name = "Takeoff"
check = "altitude"
greater_than = 100.0
"#;

        let real = toml::to_string(&config).unwrap();
        assert_eq!(real.as_str(), expected);
    }
}
