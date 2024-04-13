use soroban_sdk::{testutils::Address as _, Address, Env};

use zentra_direct_stream::{types::StreamSettings, DirectStreamContract, DirectStreamContractClient};
 
use crate::common;

mod direct_stream_contract_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/optimized/zentra_direct_stream.wasm"
    );
} 

/// Create a direct stream contract
///
/// Returns (direct_stream, underlying_token)
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `settings` - The settings for the direct_stream
pub fn create_direct_stream<'a>(
    e: &Env,
    admin: &Address,
    settings: &StreamSettings,
) -> (Address, Address) {
    let stream_address = e.register_contract(None, DirectStreamContract {});
    let (underlying_token, _) = common::create_stellar_token(e, admin);
    let stream_client: DirectStreamContractClient<'a> =
    DirectStreamContractClient::new(&e, &stream_address);
    stream_client.initialize(settings);
    return (stream_address, underlying_token);
}

/// Create a direct stream contract with the wasm contract
///
/// Returns (direct_stream, underlying_token)
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `settings` - The settings for the direct_stream
pub fn create_direct_stream_wasm<'a>(
    e: &Env,
    admin: &Address,
    settings: &StreamSettings,
) -> (Address, Address) {
    let direct_stream_address = e.register_contract_wasm(None, direct_stream_contract_wasm::WASM);
    let (underlying_token, _) = common::create_stellar_token(e, admin);
    let direct_stream_client: DirectStreamContractClient<'a> =
        DirectStreamContractClient::new(&e, &direct_stream_address);
    direct_stream_client.initialize(settings);
    return (direct_stream_address, underlying_token);
}

/// Default direct stream settings
pub fn default_stream_settings(e: &Env) -> StreamSettings {
    StreamSettings {
        admin: Address::generate(e),
        base_fee: 1_000,
    }
}
