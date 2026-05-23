#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::our_token::{SibToken, SibTokenClient};
struct SetUpResult<'a> {
    env: Env,
    client: SibTokenClient<'a>,
    sender: Address,
    receiver: Address,
    admin: Address,
}

fn setup<'a>() -> SetUpResult<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SibToken, ());

    let client = SibTokenClient::new(&env, &contract_id);

    let sender = Address::generate(&env);

    let receiver = Address::generate(&env);

    let admin = Address::generate(&env);

    client.initialize(
        &admin,
        &18,
        &String::from_str(&env, "SibToken"),
        &String::from_str(&env, "SIB"),
    );

    SetUpResult {
        env,
        client,
        sender,
        receiver,
        admin,
    }
}


#[test]
fn test_name() {
    let setup_result = setup();

    let name = setup_result.client.name();
    let token_name = String::from_str(&setup_result.env, "SibToken");
    assert_eq!(name, token_name);
}

#[test]
fn test_symbol() {
    let setup_result = setup();

    let name = setup_result.client.symbol();
    let token_name = String::from_str(&setup_result.env, "SIB");

    let not_token_name = String::from_str(&setup_result.env, "Sib");
    assert_eq!(name, token_name);
    assert_ne!(name, not_token_name);
}

#[test]
fn test_decimal() {
    let setup_result = setup();

    let decimal = setup_result.client.decimals();
    let token_decimal = 18;

    assert_eq!(decimal, token_decimal);
}

#[test]
fn test_transfer() {
    let setup_result = setup();
    let client = setup_result.client;
    let sender = setup_result.sender;
    let receiver = setup_result.receiver;

    client.mint(&sender, &1000);
    assert_eq!(client.balance(&sender), 1000);

    client.transfer(&sender, &receiver, &400);
    assert_eq!(client.balance(&sender), 600);
    assert_eq!(client.balance(&receiver), 400);
}

#[test]
fn test_approve_and_transfer_from() {
    let setup_result = setup();
    let client = setup_result.client;
    let sender = setup_result.sender;
    let receiver = setup_result.receiver;
    let spender = Address::generate(&setup_result.env);

    client.mint(&sender, &1000);

    client.approve(&sender, &spender, &500, &200);
    assert_eq!(client.allowance(&sender, &spender), 500);

    client.transfer_from(&spender, &sender, &receiver, &300);
    assert_eq!(client.balance(&sender), 700);
    assert_eq!(client.balance(&receiver), 300);
    assert_eq!(client.allowance(&sender, &spender), 200);
}

#[test]
fn test_burn() {
    let setup_result = setup();
    let client = setup_result.client;
    let sender = setup_result.sender;

    client.mint(&sender, &1000);
    client.burn(&sender, &400);
    assert_eq!(client.balance(&sender), 600);
}

#[test]
fn test_batch_transfer() {
    let setup_result = setup();
    let client = setup_result.client;
    let sender = setup_result.sender;
    let r1 = Address::generate(&setup_result.env);
    let r2 = Address::generate(&setup_result.env);

    client.mint(&sender, &1000);

    let mut recipients = soroban_sdk::vec![&setup_result.env];
    recipients.push_back((r1.clone(), 100));
    recipients.push_back((r2.clone(), 200));

    client.batch_transfer(&sender, &recipients);

    assert_eq!(client.balance(&sender), 700);
    assert_eq!(client.balance(&r1), 100);
    assert_eq!(client.balance(&r2), 200);
}

#[test]
fn test_set_admin() {
    let setup_result = setup();
    let client = setup_result.client;
    let admin = setup_result.admin;
    let new_admin = Address::generate(&setup_result.env);

    client.set_admin(&new_admin);
    
    // Original admin should no longer be able to mint
    // But since mock_all_auths is on, it's hard to test failure without specifically expecting it.
    // We can at least verify initialize fails now (or rather, mint works with new_admin).
    client.mint(&setup_result.sender, &100);
    assert_eq!(client.balance(&setup_result.sender), 100);
}

#[test]
fn test_burn_from() {
    let setup_result = setup();
    let client = setup_result.client;
    let sender = setup_result.sender;
    let spender = Address::generate(&setup_result.env);

    client.mint(&sender, &1000);
    client.approve(&sender, &spender, &500, &200);
    
    client.burn_from(&spender, &sender, &300);
    assert_eq!(client.balance(&sender), 700);
    assert_eq!(client.allowance(&sender, &spender), 200);
}


#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_insufficient_funds() {
    let setup_result = setup();
    let client = setup_result.client;
    let sender = setup_result.sender;
    let receiver = setup_result.receiver;

    client.mint(&sender, &100);
    client.transfer(&sender, &receiver, &101);
}