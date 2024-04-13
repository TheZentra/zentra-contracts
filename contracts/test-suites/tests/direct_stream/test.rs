#![cfg(test)]

extern crate std;

use std::println;

use super::testutils::{register_test_contract as register_streamtoken, StreamToken};
use soroban_sdk::{
    testutils::{Address as AddressTestTrait, Ledger},
    Address, Env,
};
use sep_41_token::testutils::MockTokenClient;

fn create_streamtoken_contract(e: &Env, admin: &Address, fee: &i128) -> StreamToken {
    let id = register_streamtoken(e);
    let streamtoken = StreamToken::new(e, id.clone());
    streamtoken.client().initialize(admin, fee);
    streamtoken
}

fn advance_ledger(e: &Env, delta: u64) {
    e.ledger().with_mut(|l| {
        l.timestamp += delta;
    });
}

struct Setup<'a> {
    env: Env,
    sender: Address,
    recipient: Address,
    token_client: MockTokenClient<'a>,
    streamtoken: StreamToken,
    stream_id: u32,
}

pub fn create_stellar_token<'a>(e: &Env, admin: &Address) -> (Address, MockTokenClient<'a>) {
    let contract_id = e.register_stellar_asset_contract(admin.clone());
    let client = MockTokenClient::new(e, &contract_id);
    (contract_id, client)
}

/// Sets up a streamtoken with -
///
impl Setup<'_> {
    fn new() -> Self {
        let e: Env = soroban_sdk::Env::default();
        let sender = Address::generate(&e);
        let recipient = Address::generate(&e);

        // the deadline is 10 seconds from now
        let timestamp = e.ledger().timestamp();
        let deadline = timestamp + 31_536_000;

        // Create the token contract
        let token_admin = Address::generate(&e);
        let (token_address, token_client) = create_stellar_token(&e, &token_admin);

        // Create the streamtokening contract
        let streamtoken = create_streamtoken_contract(&e, &token_admin, &10);

        // Mint some tokens to work with
        token_client.mock_all_auths().mint(&sender, &100_000_000);
        token_client.mock_all_auths().mint(&recipient, &80_000_000);

        let stream_id = streamtoken.client().mock_all_auths().create_range_stream(
            &sender,
            &recipient,
            &10_000_000,
            &token_address,
            &timestamp,
            &deadline,
            &true,
            &timestamp
        );

        println!("stream_id - {:?}", stream_id);

        Self {
            env: e,
            sender,
            recipient,
            token_client,
            streamtoken,
            stream_id,
        }
    }
}

#[test]
fn test_get_stream() {
    let setup = Setup::new();

    let stream = setup
        .streamtoken
        .client()
        .mock_all_auths()
        .get_stream(&setup.stream_id);

    // log the stream info
    println!("streams - {:?}", stream);

    assert_eq!(stream.deposit, 10_000_000);
    assert_eq!(stream.stop_time, 31_536_000);
    assert_eq!(stream.withdrawn, 0);
}

#[test]
fn test_streamed_amount() {
    let setup = Setup::new();
    advance_ledger(&setup.env, 31_536);

    let streamed_amount = setup.streamtoken.client().streamed_amount(&setup.stream_id);

    assert_eq!(streamed_amount, 10_000);
}

#[test]
fn test_withdraw_from_stream() {
    let setup = Setup::new();
    advance_ledger(&setup.env, 31_536);

    let old_stream_info = setup
        .streamtoken
        .client()
        .mock_all_auths()
        .get_stream(&setup.stream_id);

    assert_eq!(old_stream_info.withdrawn, 0);
    assert_eq!(
        setup.token_client.mock_all_auths().balance(&setup.recipient),
        80_000_000
    );

    setup
        .streamtoken
        .client()
        .mock_all_auths()
        .withdraw_from_stream(&setup.sender, &setup.recipient, &setup.stream_id, &10_000);

    let new_stream_info = setup
        .streamtoken
        .client()
        .mock_all_auths()
        .get_stream(&setup.stream_id);

    assert_eq!(new_stream_info.withdrawn, 10_000);
    assert_eq!(
        setup.token_client.mock_all_auths().balance(&setup.recipient),
        80_010_000
    );
}

#[test]
fn test_cancel_stream() {
    let setup = Setup::new();
    advance_ledger(&setup.env, 15_768_000);

    let old_stream_info = setup
        .streamtoken
        .client()
        .mock_all_auths()
        .get_stream(&setup.stream_id);

    assert_eq!(old_stream_info.withdrawn, 0);
    assert_eq!(old_stream_info.is_cancelled, false);
    assert_eq!(
        setup.token_client.mock_all_auths().balance(&setup.sender),
        90_000_000 - 10 // 10 is the fee
    );
    assert_eq!(
        setup.token_client.mock_all_auths().balance(&setup.recipient),
        80_000_000
    );

    setup
        .streamtoken
        .client()
        .mock_all_auths()
        .cancel_stream(&setup.sender, &setup.stream_id);

    let new_stream_info = setup
        .streamtoken
        .client()
        .mock_all_auths()
        .get_stream(&setup.stream_id);

    assert_eq!(new_stream_info.is_cancelled, true);
    assert_eq!(
        setup.token_client.mock_all_auths().balance(&setup.sender),
        95_000_000 - 10 // 10 is the fee
    );
    assert_eq!(
        setup.token_client.mock_all_auths().balance(&setup.recipient),
        85_000_000
    );
}
