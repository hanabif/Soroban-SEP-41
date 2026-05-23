use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    InsufficientFunds = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    Paused = 4,
}