use soroban_sdk::{contractclient, Address, Env};

use crate::types::{StreamSettings, DirectStreamData, Status};

#[contractclient(name = "DirectStreamClient")]
pub trait DirectStream {
    /// Setup the linear stream contract
    ///
    /// ### Arguments
    /// * `settings` - The settings for the stream
    fn initialize(e: Env, settings: StreamSettings);

    /// Get the current settings of the stream
    fn settings(e: Env) -> StreamSettings;

    /// Create a new proposal
    ///
    /// Returns the id of the new proposal
    ///
    /// ### Arguments
    /// * `creator` - The address of the account creating the proposal
    /// * `title` - The title of the proposal
    /// * `description` - The description of the proposal
    /// * `action` - The action the proposal will take if passed
    ///
    /// ### Panics
    /// If the proposal is not created successfully
    fn get_stream(e: Env, stream_id: u32) -> Option<DirectStreamData>;

    fn streamed_amount(e: Env, stream_id: u32) -> i128;

    fn stream_status(e: Env, stream_id: u32) -> Option<Status>; 

    fn create_range(
        e: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        token_address: Address,
        start_time: u64,
        stop_time: u64,
        cancellable: bool,
        cliff_time: u64,
    ) -> u32;

    fn withdraw_from_stream(
        e: Env,
        caller: Address,
        recipient: Address,
        stream_id: u32,
        amount: i128,
    );

    fn cancel_stream(e: Env, caller: Address, stream_id: u32);
}
