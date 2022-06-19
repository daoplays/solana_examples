use borsh::{BorshSerialize};
use crate::state::{State, Error, Result};

/// Determines and reports the size of greeting data.
pub fn get_state_size() -> Result<usize> {
    let encoded = State {random_numbers: [0.0; 512]}
        .try_to_vec()
        .map_err(|e| Error::SerializationError(e))?;
    Ok(encoded.len())
}
