use std::{collections::HashMap, convert::TryInto};

use crate::{upper, CheckConditionError, Error, StateCountError};
use common::{CheckIndex, CommandIndex, StateIndex};
use heapless::Vec;
use nova_software_common as common;
use std::vec::Vec as StdVec;

struct Temp<'s>(HashMap<&'s str, common::StateIndex>);

impl<'s> Temp<'s> {
    fn new(states: &'s [upper::State]) -> Self {
        Self(
            states
                .iter()
                .enumerate()
                .map(|(i, state)| {
                    let i: u8 = i.try_into().unwrap();

                    // SAFETY: `i` comes from enumerate, which only yields indices in range
                    let index = unsafe { common::StateIndex::new_unchecked(i) };
                    (state.name.as_str(), index)
                })
                .collect(),
        )
    }

    fn get_index(&self, name: &str) -> Result<common::StateIndex, Error> {
        self.0
            .get(name)
            .copied()
            .ok_or_else(|| Error::StateNotFound(name.into()))
    }
}

//When we go to a low level file, the default state must be first
pub fn verify(mid: upper::ConfigFile) -> Result<common::ConfigFile, Error> {
    if mid.states.len() == 0 {
        return Err(Error::StateCount(StateCountError::NoStates));
    }
    if mid.states.len() > u8::MAX as usize {
        return Err(Error::StateCount(StateCountError::TooManyStates(
            mid.states.len(),
        )));
    }

    let temp = Temp::new(&mid.states);

    let mut states: Vec<common::State, { common::MAX_STATES }> = mid
        .states
        .iter()
        // At this point we dont know the indices for the checks or commands so put in filler data
        .map(|_| common::State::new(Vec::new(), Vec::new(), None))
        .collect();

    let default_state = mid.default_state.map_or_else(
        // SAFETY: We have checked that there is at least one state above, so index 0 is in bounds
        || Ok(unsafe { StateIndex::new_unchecked(0) }),
        |name| temp.get_index(&name),
    )?;

    let mut dst_commands: Vec<common::Command, { common::MAX_COMMANDS }> = Vec::new();
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

    Ok(common::ConfigFile {
        default_state,
        states,
        checks: dst_checks,
        commands: dst_commands,
    })
}

// greater_than: Option<f32>,
// upper_bound: Option<f32>,
// lower_bound: Option<f32>,
// flag: Option<String>,

fn convert_check(check: &upper::Check) -> Result<common::Check, Error> {
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
    //The user only set one option, now map that to an object and state
    let condition = {
        if let Some(gt) = check.greater_than {
            common::CheckCondition::GreaterThan(gt)
        } else if let (Some(l), Some(u)) = (check.lower_bound, check.upper_bound) {
            common::CheckCondition::Between {
                upper_bound: u,
                lower_bound: l,
            }
        } else if let Some(flag) = check.flag {
            common::CheckCondition::GreaterThan(gt)
        }else {
            unreachable!()
        }
    };

    Ok(common::Check {
        object: check.object,
        condition: todo!(),
        transition: todo!(),
    })
}

#[cfg(test)]
mod tests {
    use super::common;

    /// Used for format compatibility guarantees. Call with real encoded config files once we have
    /// a stable version to maintain
    fn assert_config_eq(bytes: Vec<u8>, config: common::ConfigFile) {
        let decoded: common::ConfigFile = postcard::from_bytes(bytes.as_slice()).unwrap();
        assert_eq!(decoded, config);
    }
}
