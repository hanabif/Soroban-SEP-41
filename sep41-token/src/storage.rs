use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct AllowanceKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub live_until_ledger: u32,
}

#[contracttype]
pub enum DataKey {
    Balance(Address),
    Allowance(AllowanceKey),
    Admin,
    Name,
    Symbol,
    Decimals,
    Supply,
    IsPaused,
}


pub const INSTANCE_BUMP_AMOUNT: u32 = 34560; // ~2 days
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day

pub const PERSISTENT_BUMP_AMOUNT: u32 = 34560;
pub const PERSISTENT_LIFETIME_THRESHOLD: u32 = 17280;
