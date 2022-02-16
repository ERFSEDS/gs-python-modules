//! Responsible for converting a high level toml string into a [`ConfigFile`].
//! This module's `ConfigFile` is slightly different from nova_software_common's. This one uses
//! string names to reference states and checks, which may not be linked. This struct server as a
//! high level bridge from the automated toml code and the low-level generator. This verify step only
//! checks for valid toml. The returned [`ConfigFile`] may reference state or check names that don't
//! exist, have negative timeouts, etc. This is the job of the low level verifier to check when it
//! converts our [`ConfigFile`] to [`nova_software_common::ConfigFile`]
use serde::{Deserialize, Serialize};

pub fn verify(toml: &str) -> Result<ConfigFile, crate::Error> {
    Ok(toml::from_str(toml)?)
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ConfigFile {
    default_state: Option<String>,
    states: Vec<State>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Timeout {
    /// How long this state can execute in seconds before the rocket automatically transitions to
    /// `state`
    seconds: f32,

    /// The state to transition to when `state`
    state: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
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
#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Check {
    /// The name describing this check
    name: String,

    /// The name of the thing to be checked
    /// Currently only the strings `altitude`, `pyro1`, `pyro2`, and `pyro3` are supported, and
    /// enable specific filtering conditions
    check: String,

    /// The name of the state to transition to when when the check is tripped
    transition: Option<String>,

    /// If set, this check will execute when the value of `self.check` > the inner value
    /// Only available for `altitude` checks
    greater_than: Option<f32>,

    /// Forms a check range with `lower_bound` that checks if `check` is in a particular range
    /// Only available for `altitude` checks
    upper_bound: Option<f32>,

    /// Must be Some(...) if `upper_bound` is Some(...), and must be None if `upper_bound` is none
    lower_bound: Option<f32>,

    /// Checks if a boolean flag is set or unset
    /// The pyro values are supported
    /// `flag = "set"` or `flag = "unset"`
    flag: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Command {
    object: String,
    value: f32,
    delay: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_serialize1() {
        let expected = ConfigFile {
            default_state: Some("PowerOn".to_owned()),
            states: vec![State {
                name: "PowerOn".to_owned(),
                checks: vec![],
                //commands: vec![],
                timeout: None,
            }],
        };
        let config = r#"default_state = "PowerOn"

[[states]]
name = "PowerOn"
checks = []
"#;

        let parsed = verify(config).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn basic_serialize2() {
        let expected = ConfigFile {
            default_state: Some("PowerOn".to_owned()),
            states: vec![State {
                name: "PowerOn".to_owned(),
                timeout: None,
                checks: vec![Check {
                    name: "Takeoff".to_owned(),
                    check: "altitude".to_owned(),
                    greater_than: Some(100.0),
                    transition: None,
                    upper_bound: None,
                    flag: None,
                    lower_bound: None,
                }],
                //checks: vec![],
                //commands: vec![],
            }],
        };

        let config = r#"default_state = "PowerOn"

[[states]]
name = "PowerOn"

[[states.checks]]
name = "Takeoff"
check = "altitude"
greater_than = 100.0
"#;

        let parsed = verify(config).unwrap();
        assert_eq!(parsed, expected);
    }
}
