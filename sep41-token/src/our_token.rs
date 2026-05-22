use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::{
    error::ContractError,
    events::{Approval, Burn, Transfer},
    storage::{AllowanceKey, DataKey},
};

#[contract]
pub struct SibToken;

#[contractimpl]
impl SibToken {
    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Allowance(AllowanceKey { from, spender }))
            .unwrap_or(0)
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        live_until_ledger: u32,
    ) -> Result<(), ContractError> {
        from.require_auth();

        let from_balance = Self::balance(env.clone(), from.clone());

        if from_balance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        let key = DataKey::Allowance(AllowanceKey {
            from: from.clone(),
            spender: spender.clone(),
        });

        env.storage().persistent().set(&key, &amount);

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
        let mut from_balance = Self::balance(env.clone(), from.clone());

        let mut to_balance = Self::balance(env.clone(), to.clone());

        if from_balance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        from_balance -= amount;
        to_balance += amount;

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &from_balance,
        );
        
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()),&to_balance,
        );

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

        let key = DataKey::Allowance(AllowanceKey {
            from: from.clone(),
            spender: spender.clone(),
        });

        let allowance = env
            .storage()
            .persistent()
            .get::<DataKey, i128>(&key)
            .unwrap_or(0);

        if allowance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());

        if from_balance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        env.storage().persistent().set(&key, &(allowance - amount));

        env.storage().persistent().set(
            &DataKey::Balance(from.clone()),
            &(from_balance - amount),
        );

        env.storage().persistent().set(
            &DataKey::Balance(to.clone()),
            &(to_balance + amount),
        );

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

        let balance = Self::balance(env.clone(), from.clone());

        if balance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        env.storage().persistent().set(
            &DataKey::Balance(from.clone()),
            &(balance - amount),
        );

        let supply = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Supply)
            .unwrap_or(0);

        env.storage().persistent().set(
            &DataKey::Supply,
            &(supply - amount),
        );

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

        let key = DataKey::Allowance(AllowanceKey {
            from: from.clone(),
            spender: spender.clone(),
        });

        let allowance = env
            .storage()
            .persistent()
            .get::<DataKey, i128>(&key)
            .unwrap_or(0);

        if allowance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        let balance = Self::balance(env.clone(), from.clone());

        if balance < amount {
            return Err(ContractError::InsufficientFunds);
        }

        env.storage().persistent().set(&key, &(allowance - amount));

        env.storage().persistent().set(
            &DataKey::Balance(from.clone()),
            &(balance - amount),
        );

        let supply = env
            .storage()
            .persistent()
            .get::<_, i128>(&DataKey::Supply)
            .unwrap_or(0);

        env.storage().persistent().set(
            &DataKey::Supply,
            &(supply - amount),
        );

        Burn { from, amount }.publish(&env);

        Ok(())
    }


    pub fn decimals(_env: Env) -> u32 {
        18
    }

    pub fn name(env: Env) -> String {
        String::from_str(&env, "SibToken")
    }

    pub fn symbol(env: Env) -> String {
        String::from_str(&env, "SIB")
    }
}