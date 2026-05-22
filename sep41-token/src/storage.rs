use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct AllowanceKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
pub enum DataKey {
    Balance(Address),
    Allowance(AllowanceKey),
    Supply,
}