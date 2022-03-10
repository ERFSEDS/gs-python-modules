use std::{collections::HashMap, convert::TryInto};

use crate::{upper, CheckConditionError, Error, StateCountError};
use common::index::{self, StateTransition};
use common::index::{Check, Command, ConfigFile, State, StateIndex};
use heapless::Vec;
use nova_software_common as common;
use std::vec::Vec as StdVec;

struct Temp<'s>(HashMap<&'s str, StateIndex>);

impl<'s> Temp<'s> {
    fn new(states: &'s [upper::State]) -> Self {
        Self(
            states
                .iter()
                .enumerate()
                .map(|(i, state)| {
                    let i: u8 = i.try_into().unwrap();

                    // SAFETY: `i` comes from enumerate, which only yields indices in range
                    let index = unsafe { StateIndex::new_unchecked(i) };
                    (state.name.as_str(), index)
                })
                .collect(),
        )
    }

    fn get_index(&self, name: &str) -> Result<StateIndex, Error> {
        self.0
            .get(name)
            .copied()
            .ok_or_else(|| Error::StateNotFound(name.into()))
    }
}

//When we go to a low level file, the default state must be first
pub fn verify(mid: upper::ConfigFile) -> Result<ConfigFile, Error> {
    if mid.states.len() == 0 {
        return Err(Error::StateCount(StateCountError::NoStates));
    }
    if mid.states.len() > u8::MAX as usize {
        return Err(Error::StateCount(StateCountError::TooManyStates(
            mid.states.len(),
        )));
    }

    let temp = Temp::new(&mid.states);

    let mut states: Vec<State, { common::MAX_STATES }> = mid
        .states
        .iter()
        // At this point we dont know the indices for the checks or commands so put in filler data
        .map(|_| State::new(Vec::new(), Vec::new(), None))
        .collect();

    let default_state = mid.default_state.map_or_else(
        // SAFETY: We have checked that there is at least one state above, so index 0 is in bounds
        || Ok(unsafe { StateIndex::new_unchecked(0) }),
        |name| temp.get_index(&name),
    )?;

    /*
    let mut dst_commands: Vec<Command, { common::MAX_COMMANDS }> = Vec::new();
    let mut dst_checks: Vec<common::Check, { common::MAX_CHECKS }> = Vec::new();
    for (src_state, dst_state) in mid.states.iter().zip(states.iter_mut()) {
        for src_check in &src_state.checks {
            // Convert command to lower type
            let dst_check: common::Check = convert_check(src_check, temp);

            let index: u8 = dst_checks.len().try_into().unwrap();
            // SAFETY: `index` is the index of the to-be-pushed check therefore it is in range
            let index = unsafe { CheckIndex::new_unchecked(index) };

            // Push command and its index
            dst_checks.push(dst_check).unwrap();
            dst_state.checks.push(index).unwrap();
        }

        for src_command in &src_state.commands {
            // Convert command to lower type
            let dst_command: common::Command = src_command.try_into().unwrap();

            let index: u8 = dst_commands.len().try_into().unwrap();
            // SAFETY: `index` is the index of the to-be-pushed command therefore it is in range
            let index = unsafe { CommandIndex::new_unchecked(index) };

            // Push command and its index
            dst_commands.push(dst_command).unwrap();
            dst_state.commands.push(index).unwrap();
        }
    }
    */

    Ok(ConfigFile {
        default_state,
        states,
    })
}

// greater_than: Option<f32>,
// upper_bound: Option<f32>,
// lower_bound: Option<f32>,
// flag: Option<String>,

fn convert_check(check: &upper::Check, temp: Temp<'_>) -> Result<Check, Error> {
    if check.upper_bound.is_some() && check.lower_bound.is_none()
        || check.upper_bound.is_none() && check.lower_bound.is_some()
    {
        panic!(
            "Unmatched bound! if one of `lower_bound` or `higher_bound` is used, both must be set"
        );
    }
    let mut count = 0;
    if check.greater_than.is_some() {
        count += 1;
    }
    if check.upper_bound.is_some() && check.lower_bound.is_some() {
        count += 1;
    }
    if check.flag.is_some() {
        count += 1;
    }
    if count == 0 {
        return Err(Error::CheckConditionError(CheckConditionError::NoCondition));
    }
    if count > 1 {
        return Err(Error::CheckConditionError(
            CheckConditionError::TooManyConditions(count),
        ));
    }

    enum CheckKind {
        Apogee,
        Altitude,
        Pyro1Continuity,
        Pyro2Continuity,
        Pyro3Continuity,
    }
    let check_kind = match check.check.as_str() {
        "apogee" => CheckKind::Apogee,
        "altitude" => CheckKind::Altitude,
        "pyro1_continuity" => CheckKind::Pyro1Continuity,
        "pyro2_continuity" => CheckKind::Pyro2Continuity,
        "pyro3_continuity" => CheckKind::Pyro3Continuity,
        other => panic!("Bad check {}", other), // TODO: Better error handeling
    };

    pub enum CheckCondition {
        FlagEq(bool),
        // Equals { value: f32 },
        GreaterThan(f32),
        LessThan(f32),
        Between { upper_bound: f32, lower_bound: f32 },
    }

    //The user only set one option, now map that to an object and state
    let condition = {
        if let Some(gt) = check.greater_than {
            CheckCondition::GreaterThan(gt)
        } else if let (Some(u), Some(l)) = (check.upper_bound, check.lower_bound) {
            CheckCondition::Between {
                upper_bound: u,
                lower_bound: l,
            }
        } else if let Some(flag) = &check.flag {
            match flag.as_str() {
                "set" => CheckCondition::FlagEq(true),
                "unset" => CheckCondition::FlagEq(false),
                _ => panic!("Unknown flag: {}", flag), // TODO: Better error handeling
            }
        } else {
            unreachable!()
        }
    };

    use common::{CheckData, FloatCondition, NativeFlagCondition, PyroContinuityCondition};
    // Perform type checking on kind and condition
    let data = match check_kind {
        CheckKind::Apogee => match condition {
            CheckCondition::FlagEq(val) => CheckData::ApogeeFlag(NativeFlagCondition(val)),
            _ => panic!(),
        },
        CheckKind::Altitude => match condition {
            CheckCondition::Between {
                upper_bound,
                lower_bound,
            } => CheckData::Altitude(FloatCondition::Between {
                upper_bound,
                lower_bound,
            }),
            CheckCondition::GreaterThan(val) => {
                CheckData::Altitude(FloatCondition::GreaterThan(val))
            }
            CheckCondition::LessThan(val) => CheckData::Altitude(FloatCondition::LessThan(val)),
            _ => panic!(),
        },
        CheckKind::Pyro1Continuity => match condition {
            CheckCondition::FlagEq(val) => CheckData::Pyro1Continuity(PyroContinuityCondition(val)),
            _ => panic!(),
        },
        CheckKind::Pyro2Continuity => match condition {
            CheckCondition::FlagEq(val) => CheckData::Pyro2Continuity(PyroContinuityCondition(val)),
            _ => panic!(),
        },
        CheckKind::Pyro3Continuity => match condition {
            CheckCondition::FlagEq(val) => CheckData::Pyro3Continuity(PyroContinuityCondition(val)),
            _ => panic!(),
        },
    };

    let transition = match &check.transition {
        Some(state) => Some(temp.get_index(state.as_str())?),
        None => None,
    };

    let abort = match &check.abort {
        Some(state) => Some(temp.get_index(state.as_str())?),
        None => None,
    };

    let transition = match (transition, abort) {
        (Some(t), None) => Some(StateTransition::Transition(t)),
        (None, Some(a)) => Some(StateTransition::Abort(a)),
        (None, None) => None,
        (Some(_), Some(_)) => panic!(),
    };

    Ok(index::Check::new(data, transition))
}

#[cfg(test)]
mod tests {
    use super::common;

    /// Used for format compatibility guarantees. Call with real encoded config files once we have
    /// a stable version to maintain
    fn assert_config_eq(bytes: Vec<u8>, config: common::index::ConfigFile) {
        let decoded: common::index::ConfigFile = postcard::from_bytes(bytes.as_slice()).unwrap();
        assert_eq!(decoded, config);
    }
}
