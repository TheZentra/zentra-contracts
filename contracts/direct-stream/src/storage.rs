use soroban_sdk::{
    contracttype, unwrap::UnwrapOptimized, Address, Env, IntoVal, Symbol, TryFromVal, Val,
};

use crate::{
    constants::ONE_DAY_LEDGERS,
    types::{StreamSettings, DirectStreamData},
};

const SETTINGS_KEY: &str = "Settings";
const IS_INIT_KEY: &str = "IsInit";
const STREAM_ID_KEY: &str = "StreamId";

const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + ONE_DAY_LEDGERS; // ~ 31 days

const LEDGER_THRESHOLD_USER: u32 = ONE_DAY_LEDGERS * 100; // ~ 100 days
const LEDGER_BUMP_USER: u32 = LEDGER_THRESHOLD_USER + 20 * ONE_DAY_LEDGERS; // ~ 120 days

//********** Storage Keys **********//

// Key for storing Voter's decision
#[derive(Clone)]
#[contracttype]
pub struct VoterStatusKey {
    pub proposal_id: u32,
    pub voter: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum StreamDataKey {
    // A map of the stream to the id
    Streams(u32),
    // A map of the fee to the asset address
    AssetFee(Address),
}

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_persistent_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>>(
    e: &Env,
    key: &K,
    default: V,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().persistent().get::<K, V>(key) {
        e.storage()
            .persistent()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default
    }
}

/********** Instance **********/

/// Check if the contract has been initialized
pub fn get_is_init(e: &Env) -> bool {
    e.storage().instance().has(&Symbol::new(e, IS_INIT_KEY))
}

/// Set the contract as initialized
pub fn set_is_init(e: &Env) {
    e.storage()
        .instance()
        .set::<Symbol, bool>(&Symbol::new(e, IS_INIT_KEY), &true);
}

/// Set the contract settings
///
/// ### Arguments
/// * `settings` - The contract settings
pub fn set_settings(e: &Env, settings: &StreamSettings) {
    e.storage()
        .instance()
        .set::<Symbol, StreamSettings>(&Symbol::new(&e, SETTINGS_KEY), &settings);
}

/// Get the contract settings
pub fn get_settings(e: &Env) -> StreamSettings {
    e.storage()
        .instance()
        .get::<Symbol, StreamSettings>(&Symbol::new(&e, SETTINGS_KEY))
        .unwrap_optimized()
}

/********** Persistent **********/

/// Set the next stream id and bump if necessary
///
/// ### Arguments
/// * `stream_id` - The new stream_id
pub fn set_next_stream_id(e: &Env, stream_id: u32) {
    let key = Symbol::new(&e, STREAM_ID_KEY);
    e.storage()
        .persistent()
        .set::<Symbol, u32>(&key, &stream_id);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_USER, LEDGER_BUMP_USER);
}

/// Get the current stream id
pub fn get_next_stream_id(e: &Env) -> u32 {
    let key = Symbol::new(&e, STREAM_ID_KEY);
    get_persistent_default::<Symbol, u32>(&e, &key, 1_u32, LEDGER_THRESHOLD_USER, LEDGER_BUMP_USER)
}

pub fn get_stream(e: &Env, stream_id: &u32) -> Option<DirectStreamData> {
    let key = StreamDataKey::Streams(stream_id.clone());
    e.storage()
        .persistent()
        .get::<StreamDataKey, DirectStreamData>(&key)
}

pub fn set_stream(e: &Env, stream_id: &u32, stream: &DirectStreamData) {
    let key = StreamDataKey::Streams(stream_id.clone());
    e.storage()
        .persistent()
        .set::<StreamDataKey, DirectStreamData>(&key, &stream);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD_USER, LEDGER_BUMP_USER);
}

/********** Temporary **********/

/***** Stream Config *****/

/// Fetch stream config at `proposal_id`
///
/// ### Arguments
/// * `proposal_id` - The id of the proposal to fetch
pub fn get_asset_fee(e: &Env, token_address: &Address) -> Option<i128> {
    let key = StreamDataKey::AssetFee(token_address.clone());
    e.storage()
        .temporary()
        .get::<StreamDataKey, i128>(&key)
}

pub fn set_asset_fee(e: &Env, token_address: Address, fee: &i128) {
    let key = StreamDataKey::AssetFee(token_address);
    e.storage()
        .temporary()
        .set::<StreamDataKey, i128>(&key, &fee);
    e.storage()
        .temporary()
        .extend_ttl(&key, LEDGER_THRESHOLD_USER, LEDGER_BUMP_USER);
}


