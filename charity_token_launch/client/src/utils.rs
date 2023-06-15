use borsh::{BorshSerialize};
use crate::state::{ICOData, Error, Result};

/// Determines and reports the size of greeting data.
pub fn get_state_size() -> usize {
    let encoded = ICOData {charity_totals: [0; 7], donated_total : 0, paid_total : 0, n_donations : 0}
        .try_to_vec().unwrap();

    encoded.len()
}
