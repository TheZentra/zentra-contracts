#[cfg(test)]
use sep_41_token::testutils::MockTokenClient;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Symbol, IntoVal,
};
use test_suites::{
    direct_stream::{create_direct_stream, default_stream_settings},
    env::EnvTestUtils, ONE_DAY_LEDGERS,
};
use zentra_direct_stream::DirectStreamContractClient;

#[test]
fn test_withdraw() {
    let e = Env::default();
    e.mock_all_auths();
    e.set_default_info();
    let deadline = e.ledger().timestamp() + (365 * ONE_DAY_LEDGERS) as u64;

    let yosemite = Address::generate(&e);
    let everest = Address::generate(&e);
    let settings = default_stream_settings(&e);
    let (stream_address, token_address) =
        create_direct_stream(&e, &settings.admin, &settings);
    let token_client = MockTokenClient::new(&e, &token_address);
    let stream_client = DirectStreamContractClient::new(&e, &stream_address);

    token_client.mint(&yosemite, &100_000_000);
    token_client.mint(&everest, &100_000_000);

    let stream_amount = 10_000_000_i128;
    let withdraw_amount = 100_000_i128;

    // create a direct range stream
    let stream_id = stream_client.create_range(
        &yosemite,
        &everest,
        &stream_amount,
        &token_address,
        &e.ledger().timestamp(),
        &deadline,
        &true,
        &e.ledger().timestamp(),
    );

    // advance ledger
    e.jump(5 * ONE_DAY_LEDGERS);

    let amount_streamed = stream_client.streamed_amount(&stream_id);

    println!("amount_streamed - {:?}", amount_streamed);

    // withdraw from the stream 
    stream_client.withdraw(&everest, &everest, &stream_id, &withdraw_amount);

    // verify auths
    assert_eq!(
        e.auths()[0],
        (
            everest.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    stream_address.clone(),
                    Symbol::new(&e, "withdraw"),
                    vec![
                        &e,
                        everest.to_val(),
                        everest.to_val(),
                        stream_id.into_val(&e),
                        withdraw_amount.into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // verify chain results
    let stream = stream_client.get_stream(&stream_id).unwrap();

    assert_eq!(stream.withdrawn, withdraw_amount);
}
