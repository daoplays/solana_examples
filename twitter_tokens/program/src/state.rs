use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
};
use solana_security_txt::security_txt;



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct UserData {
    pub account_key : Pubkey,
    pub last_time : i64,
    pub follow : bool
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct IDMap {
    pub twitter_id : u64,
    pub error_code : u8
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct RewardMark {
    pub mark : bool
}

/// Determines and reports the size of user data.
pub fn get_user_data_size() -> usize {
    let encoded = UserData {account_key: solana_program::system_program::id(), last_time: 0, follow: false}
        .try_to_vec().unwrap();

    encoded.len()
}

pub fn get_id_map_size() -> usize {
    let encoded = IDMap {twitter_id: 0, error_code: 0}
        .try_to_vec().unwrap();

    encoded.len()
}

pub fn get_mark_size() -> usize {
    let encoded = RewardMark {mark: false}
        .try_to_vec().unwrap();

    encoded.len()
}

security_txt! {
    // Required fields
    name: "Example",
    project_url: "http://example.com",
    contacts: "email:example@example.com,link:https://example.com/security,discord:example#1234",
    policy: "https://github.com/solana-labs/solana/blob/master/SECURITY.md",

    // Optional Fields
    preferred_languages: "en,de",
    source_code: "https://github.com/example/example2",
    encryption: "
-----BEGIN PGP PUBLIC KEY BLOCK-----
Comment: Alice's OpenPGP certificate
Comment: https://www.ietf.org/id/draft-bre-openpgp-samples-01.html

mDMEXEcE6RYJKwYBBAHaRw8BAQdArjWwk3FAqyiFbFBKT4TzXcVBqPTB3gmzlC/U
b7O1u120JkFsaWNlIExvdmVsYWNlIDxhbGljZUBvcGVucGdwLmV4YW1wbGU+iJAE
ExYIADgCGwMFCwkIBwIGFQoJCAsCBBYCAwECHgECF4AWIQTrhbtfozp14V6UTmPy
MVUMT0fjjgUCXaWfOgAKCRDyMVUMT0fjjukrAPoDnHBSogOmsHOsd9qGsiZpgRnO
dypvbm+QtXZqth9rvwD9HcDC0tC+PHAsO7OTh1S1TC9RiJsvawAfCPaQZoed8gK4
OARcRwTpEgorBgEEAZdVAQUBAQdAQv8GIa2rSTzgqbXCpDDYMiKRVitCsy203x3s
E9+eviIDAQgHiHgEGBYIACAWIQTrhbtfozp14V6UTmPyMVUMT0fjjgUCXEcE6QIb
DAAKCRDyMVUMT0fjjlnQAQDFHUs6TIcxrNTtEZFjUFm1M0PJ1Dng/cDW4xN80fsn
0QEA22Kr7VkCjeAEC08VSTeV+QFsmz55/lntWkwYWhmvOgE=
=iIGO
-----END PGP PUBLIC KEY BLOCK-----
",
    auditors: "None",
    acknowledgements: "
The following hackers could've stolen all our money but didn't:
- Neodyme
"
}