#[cfg(test)]
use zentra_direct_stream::DirectStreamContractClient;
use soroban_sdk::Env;
use test_suites::{
    env::EnvTestUtils,
    direct_stream::{create_direct_stream, default_stream_settings},
};

#[test]
fn test_initialize_sets_storage() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let settings = default_stream_settings(&e);
    let (stream_address, _) = create_direct_stream(&e, &settings.admin, &settings);
    let stream_client = DirectStreamContractClient::new(&e, &stream_address);

    let result = stream_client.settings();
    assert_eq!(result.admin, settings.admin);
    assert_eq!(result.base_fee, settings.base_fee);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_initalize_already_initalized() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let settings = default_stream_settings(&e);
    let (stream_address, _) = create_direct_stream(&e, &settings.admin, &settings);
    let governor_client = DirectStreamContractClient::new(&e, &stream_address);

    governor_client.initialize(&settings);
}
