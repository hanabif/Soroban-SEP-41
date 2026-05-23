use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

use crate::{
    error::ContractError,
    events::{Approval, Burn, Transfer, Mint},
    storage::{AllowanceKey, DataKey, AllowanceValue, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD},
};

#[contract]
pub struct SibToken;

#[contractimpl]
impl SibToken {
    /// Initialize the contract with metadata and admin.
    pub fn initialize(
        env: Env,
        admin: Address,
        decimals: u32,
        name: String,
        symbol: String,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        Ok(())
    }

    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), ContractError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(ContractError::Unauthorized)?;
        admin.require_auth();

        if env.storage().instance().get(&DataKey::IsPaused).unwrap_or(false) {
            return Err(ContractError::Paused);
        }

        let mut balance = read_balance(&env, to.clone());
        balance += amount;
        write_balance(&env, to.clone(), balance);

        let mut supply = read_supply(&env);
        supply += amount;
        write_supply(&env, supply);

        Mint {
            admin,
            to,
            amount,
        }
        .publish(&env);

        Ok(())
    }

    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), ContractError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(ContractError::Unauthorized)?;
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        Ok(())
    }

    pub fn set_pause(env: Env, paused: bool) -> Result<(), ContractError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(ContractError::Unauthorized)?;
        admin.require_auth();

        env.storage().instance().set(&DataKey::IsPaused, &paused);
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        Ok(())
    }



    pub fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, id)
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&env, from, spender).amount
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        live_until_ledger: u32,
    ) -> Result<(), ContractError> {
        from.require_auth();

        let allowance = AllowanceValue {
            amount,
            live_until_ledger,
        };

        write_allowance(&env, from.clone(), spender.clone(), allowance);

        Approval {
            from,
            spender,
            amount,
            live_until_ledger,
        }
        .publish(&env);

        Ok(())
    }

    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        from.require_auth();
        
        do_transfer(&env, from.clone(), to.clone(), amount)?;

        Transfer {
            from,
            to,
            amount,
        }
        .publish(&env);

        Ok(())
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        spender.require_auth();

        let mut allowance = read_allowance(&env, from.clone(), spender.clone());

        if allowance.amount < amount {
            return Err(ContractError::InsufficientFunds);
        }

        if allowance.live_until_ledger < env.ledger().sequence() && allowance.amount > 0 {
             return Err(ContractError::InsufficientFunds);
        }

        allowance.amount -= amount;
        write_allowance(&env, from.clone(), spender.clone(), allowance);

        do_transfer(&env, from.clone(), to.clone(), amount)?;

        Transfer {
            from,
            to,
            amount,
        }
        .publish(&env);

        Ok(())
    }

    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), ContractError> {
        from.require_auth();

        if env.storage().instance().get(&DataKey::IsPaused).unwrap_or(false) {
            return Err(ContractError::Paused);
        }

        let mut balance = read_balance(&env, from.clone());
        if balance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        balance -= amount;
        write_balance(&env, from.clone(), balance);

        let mut supply = read_supply(&env);
        supply -= amount;
        write_supply(&env, supply);

        Burn { from, amount }.publish(&env);

        Ok(())
    }

    pub fn burn_from(
        env: Env,
        spender: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        spender.require_auth();

        if env.storage().instance().get(&DataKey::IsPaused).unwrap_or(false) {
            return Err(ContractError::Paused);
        }

        let mut allowance = read_allowance(&env, from.clone(), spender.clone());
        if allowance.amount < amount {
            return Err(ContractError::InsufficientFunds);
        }


        allowance.amount -= amount;
        write_allowance(&env, from.clone(), spender.clone(), allowance);

        let mut balance = read_balance(&env, from.clone());
        if balance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        balance -= amount;
        write_balance(&env, from.clone(), balance);

        let mut supply = read_supply(&env);
        supply -= amount;
        write_supply(&env, supply);

        Burn { from, amount }.publish(&env);

        Ok(())
    }

    pub fn decimals(env: Env) -> u32 {
        read_decimals(&env)
    }

    pub fn name(env: Env) -> String {
        read_name(&env)
    }

    pub fn symbol(env: Env) -> String {
        read_symbol(&env)
    }

    /// Creative Feature: Batch Transfer
    /// Allows sending tokens to multiple recipients in a single transaction.
    /// Gast-optimized by reducing individual require_auth calls and storage overhead.
    pub fn batch_transfer(
        env: Env,
        from: Address,
        recipients: Vec<(Address, i128)>,
    ) -> Result<(), ContractError> {
        from.require_auth();

        let mut total_amount: i128 = 0;
        for item in recipients.clone().iter() {
            total_amount += item.1;
        }

        let mut from_balance = read_balance(&env, from.clone());
        if from_balance < total_amount {
            return Err(ContractError::InsufficientFunds);
        }

        from_balance -= total_amount;
        write_balance(&env, from.clone(), from_balance);

        for item in recipients.iter() {
            let to = item.0;
            let amount = item.1;

            let mut to_balance = read_balance(&env, to.clone());
            to_balance += amount;
            write_balance(&env, to.clone(), to_balance);

            Transfer {
                from: from.clone(),
                to,
                amount,
            }
            .publish(&env);
        }

        Ok(())
    }
}

// --- Internal Helpers ---

fn read_balance(env: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    let balance = env.storage().persistent().get(&key).unwrap_or(0);
    if balance > 0 {
        env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }
    balance
}

fn write_balance(env: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr);
    env.storage().persistent().set(&key, &amount);
    env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

fn read_allowance(env: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceKey { from, spender });
    let val = env.storage().persistent().get(&key).unwrap_or(AllowanceValue {
        amount: 0,
        live_until_ledger: 0,
    });
    if val.amount > 0 {
        env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    }
    val
}

fn write_allowance(env: &Env, from: Address, spender: Address, val: AllowanceValue) {
    let key = DataKey::Allowance(AllowanceKey { from, spender });
    env.storage().persistent().set(&key, &val);
    env.storage().persistent().extend_ttl(&key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

fn read_supply(env: &Env) -> i128 {
    let supply = env.storage().instance().get(&DataKey::Supply).unwrap_or(0);
    env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    supply
}

fn write_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::Supply, &amount);
    env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn read_decimals(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Decimals).unwrap_or(18)
}

fn read_name(env: &Env) -> String {
    env.storage().instance().get(&DataKey::Name).unwrap_or(String::from_str(env, "SibToken"))
}

fn read_symbol(env: &Env) -> String {
    env.storage().instance().get(&DataKey::Symbol).unwrap_or(String::from_str(env, "SIB"))
}

fn do_transfer(env: &Env, from: Address, to: Address, amount: i128) -> Result<(), ContractError> {
    if env.storage().instance().get(&DataKey::IsPaused).unwrap_or(false) {
        return Err(ContractError::Paused);
    }

    if from == to {
        return Ok(());
    }

    let mut from_balance = read_balance(env, from.clone());
    if from_balance < amount {
        return Err(ContractError::InsufficientFunds);
    }

    let mut to_balance = read_balance(env, to.clone());

    from_balance -= amount;
    to_balance += amount;

    write_balance(env, from, from_balance);
    write_balance(env, to, to_balance);

    Ok(())
}
