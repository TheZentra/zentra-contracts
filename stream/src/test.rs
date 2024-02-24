#![cfg(test)]

extern crate std;

use super::testutils::{register_test_contract as register_streamtoken, StreamToken};
use soroban_sdk::{
    testutils::{Address as AddressTestTrait, Ledger},
    token, Address, Env,
};

fn create_streamtoken_contract(e: &Env, token: &Address) -> StreamToken {
    let id = register_streamtoken(e);
    let streamtoken = StreamToken::new(e, id.clone());
    streamtoken.client().initialize(token);
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
    token: token::Client<'a>,
    streamtoken: StreamToken,
    stream_id: u32,
}

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(e, &contract_address),
        token::StellarAssetClient::new(e, &contract_address),
    )
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
        let (token, token_admin) = create_token_contract(&e, &token_admin);

        // Create the streamtokening contract
        let streamtoken = create_streamtoken_contract(&e, &token.address);

        // Mint some tokens to work with
        token_admin.mock_all_auths().mint(&sender, &100_000_000);
        token_admin.mock_all_auths().mint(&recipient, &80_000_000);

        let stream_id = streamtoken.client().mock_all_auths().create_stream(
            &sender,
            &recipient,
            &10_000_000,
            &token.address,
            &timestamp,
            &deadline,
        );

        Self {
            env: e,
            sender,
            recipient,
            token,
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
fn test_get_stream_by_user() {
    let setup = Setup::new();

    let stream = setup
        .streamtoken
        .client()
        .mock_all_auths()
        .get_streams_by_user(&setup.sender);

    assert_eq!(stream.len(), 1);
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
        setup.token.mock_all_auths().balance(&setup.recipient),
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
        setup.token.mock_all_auths().balance(&setup.recipient),
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
        setup.token.mock_all_auths().balance(&setup.sender),
        90_000_000
    );
    assert_eq!(
        setup.token.mock_all_auths().balance(&setup.recipient),
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
        setup.token.mock_all_auths().balance(&setup.sender),
        95_000_000
    );
    assert_eq!(
        setup.token.mock_all_auths().balance(&setup.recipient),
        85_000_000
    );
}
