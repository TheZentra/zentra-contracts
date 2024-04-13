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
fn test_create_range() {
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

    // verify auths
    assert_eq!(
        e.auths()[0],
        (
            yosemite.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    stream_address.clone(),
                    Symbol::new(&e, "create_range"),
                    vec![
                        &e,
                        yosemite.to_val(),
                        everest.to_val(),
                        stream_amount.into_val(&e),
                        token_address.to_val(),
                        e.ledger().timestamp().into_val(&e),
                        deadline.into_val(&e),
                        true.into_val(&e),
                        e.ledger().timestamp().into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token_address.clone(),
                        Symbol::new(&e, "transfer"),
                        vec![
                            &e,
                            yosemite.to_val(),
                            stream_address.to_val(),
                            stream_amount.into_val(&e)
                        ]
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )
    );

    // verify chain results
    let stream = stream_client.get_stream(&stream_id).unwrap();
    assert_eq!(stream.deposit, 10_000_000);
    assert_eq!(stream.stop_time, deadline);
    assert_eq!(stream.withdrawn, 0);
    assert_eq!(stream.is_cancelled, false);
    assert_eq!(stream.is_cancellable, true);
    assert_eq!(stream.is_depleted, false);
    assert_eq!(stream.refunded, 0);
}
