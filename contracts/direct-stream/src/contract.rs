use sep_41_token::TokenClient;
use soroban_fixed_point_math::{FixedPoint, STROOP};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env,
};

use crate::{
    direct_stream::DirectStream,
    errors::StreamError,
    events::StreamEvents,
    storage,
    types::{DirectStreamData, Status, StreamSettings},
};

fn get_streamed_amount(e: &Env, stream: &DirectStreamData) -> i128 {
    let current_ledger_time = e.ledger().timestamp();

    // if the cliff time is in the future, return 0
    if stream.cliff_time > current_ledger_time {
        return 0;
    }

    // if we have gotten to the end time return the total deposited amount
    if current_ledger_time >= stream.stop_time {
        return stream.deposit;
    }

    let start_time = stream.start_time as i128;
    let elapsed_time = (current_ledger_time as i128 - start_time) as i128;
    let total_time = (stream.stop_time as i128 - start_time) as i128;

    let elapsed_time_percent = elapsed_time
        .fixed_div_floor(total_time, STROOP as i128)
        .unwrap();

    let streamed_amount = stream
        .deposit
        .fixed_mul_floor(elapsed_time_percent, STROOP as i128)
        .unwrap();

    // This is to prevent a bug from happening where the streamed amount is greater than the deposit
    // [TODO] Test this extensively to make sure it works as expected and decide whether to use amount deposited or withdrawn.
    if streamed_amount > stream.deposit {
        return stream.deposit;
    }

    streamed_amount
}

/*
Modifiers for the contract
*/
fn require_sender_or_recipient(stream: &DirectStreamData, user: &Address) {
    let sender = &(stream.sender);
    let recipient = &(stream.recipient);
    assert!(
        user == sender || user == recipient,
        "only sender or recipient can call this function"
    );
}

fn require_recipient(stream: &DirectStreamData, user: &Address) {
    let recipient = &(stream.recipient);
    assert!(
        user == recipient,
        "only recipient can receive the streamed amount"
    );
}

// Transfer tokens from the contract to the recipient
fn transfer(e: &Env, from: &Address, to: &Address, amount: &i128, token_address: &Address) {
    let token_client = TokenClient::new(e, &token_address);
    token_client.transfer(from, to, amount);
}

#[contract]
pub struct DirectStreamContract;

#[contractimpl]
#[allow(clippy::needless_pass_by_value)]
impl DirectStream for DirectStreamContract {
    fn initialize(e: Env, settings: StreamSettings) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, StreamError::AlreadyInitializedError);
        }

        storage::set_is_init(&e);
        storage::set_settings(&e, &settings);
        storage::extend_instance(&e);
    }

    fn settings(e: Env) -> StreamSettings {
        storage::get_settings(&e)
    }

    ///

    fn get_stream(e: Env, stream_id: u32) -> Option<DirectStreamData> {
        storage::get_stream(&e, &stream_id)
    }

    /// Returns the amount of tokens that have already been released to the recipient.
    /// Panics if the id does not point to a valid stream.
    /// @param stream_id The id of the stream
    /// @param who The address of the caller
    /// @return The amount of tokens that have already been released
    fn streamed_amount(e: Env, stream_id: u32) -> i128 {
        let stream = storage::get_stream(&e, &stream_id);

        if stream.is_none() {
            panic_with_error!(&e, StreamError::StreamDoesNotExist);
        } else {
            get_streamed_amount(&e, &stream.unwrap_optimized())
        }
    }

    /// Returns the status of the stream.
    fn status(e: Env, stream_id: u32) -> Option<Status> {
        let stream_resp = storage::get_stream(&e, &stream_id);

        if stream_resp.is_none() {
            return None;
        }

        let stream = stream_resp.unwrap_optimized();

        let current_ledger_time = e.ledger().timestamp();

        if stream.is_depleted {
            return Some(Status::Depleted);
        }

        if stream.is_cancelled {
            return Some(Status::Cancelled);
        }

        if current_ledger_time < stream.start_time {
            return Some(Status::Pending);
        }

        if current_ledger_time < stream.cliff_time {
            return Some(Status::Cliff);
        }

        let streamed_amount = get_streamed_amount(&e, &stream);

        if streamed_amount >= stream.deposit {
            return Some(Status::Completed);
        }

        Some(Status::Active)
    }

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
    ) -> u32 {
        sender.require_auth();
        assert!(amount > 0, "amount is zero or negative");
        assert!(
            start_time >= e.ledger().timestamp(),
            "start time before current ledger timestamp"
        );
        assert!(cliff_time >= start_time, "cliff time before the start time");
        assert!(stop_time > start_time, "stop time before the start time");
        assert!(stop_time > cliff_time, "stop time before the cliff time");

        let stream_id = storage::get_next_stream_id(&e);
        let fee = storage::get_asset_fee(&e, &token_address).unwrap_or(0);

        let total_amount = amount + fee;

        // transfer the token to the contract
        transfer(
            &e,
            &sender,
            &e.current_contract_address(),
            &total_amount,
            &token_address,
        );

        let stream = DirectStreamData {
            id: stream_id.clone(),
            sender: sender.clone(),
            recipient: recipient.clone(),
            start_time: start_time,
            stop_time: stop_time,
            cliff_time: cliff_time,
            deposit: amount,
            is_cancelled: false,
            is_cancellable: cancellable,
            is_depleted: false,
            refunded: 0,
            withdrawn: 0,
            token_address: token_address.clone(),
        };

        let next_stream_id = stream_id + 1;

        storage::set_stream(&e, &stream_id, &stream);
        storage::set_next_stream_id(&e, next_stream_id);

        StreamEvents::stream_created(
            &e,
            stream_id.clone(),
            sender,
            recipient,
            amount,
            token_address,
            start_time,
            stop_time,
        );
        stream_id
    }

    fn withdraw(
        e: Env,
        caller: Address,
        recipient: Address,
        stream_id: u32,
        amount: i128,
    ) {
        caller.require_auth();
        assert!(amount > 0, "amount is zero or negative");
        assert!(
            recipient != e.current_contract_address(),
            "stream to the contract itself"
        );

        let stream_resp = storage::get_stream(&e, &stream_id);

        if stream_resp.is_none() {
            panic_with_error!(&e, StreamError::StreamDoesNotExist);
        }

        let mut stream = stream_resp.unwrap_optimized();

        require_sender_or_recipient(&stream, &caller);
        require_recipient(&stream, &recipient);

        let streamed_amount = get_streamed_amount(&e, &stream);
        if amount > streamed_amount {
            panic_with_error!(&e, StreamError::ExceedsStreamedAmount);
        };

        stream.withdrawn = stream.withdrawn + amount;

        storage::set_stream(&e, &stream_id, &stream);

        transfer(
            &e,
            &e.current_contract_address(),
            &recipient,
            &amount,
            &stream.token_address,
        );

        StreamEvents::stream_withdrawn(&e, recipient, stream_id, amount);
    }

    /// Cancels the stream and transfers the tokens back on a pro rata basis.
    /// Throws if the id does not point to a valid stream.
    /// Throws if the caller is not the sender or the recipient of the stream.
    /// Throws if there is a token transfer failure.
    /// @param stream_id The id of the stream to cancel.
    /// @return bool true=success, otherwise false.
    fn cancel(e: Env, caller: Address, stream_id: u32) {
        caller.require_auth();
        let stream = storage::get_stream(&e, &stream_id);

        if stream.is_none() {
            panic_with_error!(&e, StreamError::StreamDoesNotExist);
        }

        let mut stream = stream.unwrap_optimized();
        require_sender_or_recipient(&stream, &caller);

        let streamed_amount = get_streamed_amount(&e, &stream);
        let recipient_balance = streamed_amount - stream.withdrawn;
        let sender_balance = stream.deposit - streamed_amount;

        stream.is_cancelled = true;

        storage::set_stream(&e, &stream_id, &stream);

        if recipient_balance > 0 {
            transfer(
                &e,
                &e.current_contract_address(),
                &stream.recipient,
                &recipient_balance,
                &stream.token_address,
            );
        }
        if sender_balance > 0 {
            transfer(
                &e,
                &e.current_contract_address(),
                &stream.sender,
                &sender_balance,
                &stream.token_address,
            );
        }

        StreamEvents::stream_cancelled(&e, stream_id);
    }
}
