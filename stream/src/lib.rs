#![no_std]
use fixed_point_math::{FixedPoint, STROOP};
use soroban_sdk::{contract, contractimpl, contracttype, log, token, Address, Env, String, Vec};

mod events;
mod test;
mod testutils;

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_EXTEND_TTL_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_EXTEND_TTL_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const PERSISTENT_EXTEND_TTL_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_EXTEND_TTL_AMOUNT - DAY_IN_LEDGERS;

#[derive(Clone, Debug)]
#[contracttype]
pub struct Stream {
    pub id: u32,
    pub sender: Address,
    pub recipient: Address,
    pub start_time: u64,
    pub stop_time: u64,
    pub deposit: i128,
    pub withdrawn: i128,
    pub is_cancelled: bool,
    pub token_address: Address,
    pub token_symbol: String,
    pub token_decimals: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Token,
    NextStreamId,
    Streams(u32),
    UserStreams(Address),
}

/*
Getter Functions
*/
fn get_ledger_timestamp(e: &Env) -> u64 {
    e.ledger().timestamp()
}

fn get_stream(e: &Env, stream_id: &u32) -> Stream {
    e.storage()
        .persistent()
        .get::<_, Stream>(&DataKey::Streams(stream_id.clone()))
        .expect("invalid stream id")
}

fn get_streams_by_user(e: &Env, who: &Address) -> Vec<Stream> {
    e.storage()
        .persistent()
        .get::<_, Vec<Stream>>(&DataKey::UserStreams(who.clone()))
        .unwrap_or(Vec::new(&e))
}

fn get_next_stream_id(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get::<_, u32>(&DataKey::NextStreamId)
        .expect("not initialized")
}

fn get_token(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<_, Address>(&DataKey::Token)
        .expect("not initialized")
}

fn get_streamed_amount(e: &Env, stream: &Stream) -> i128 {
    let current_ledger_time = get_ledger_timestamp(&e);

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

    if streamed_amount > stream.deposit {
        stream.deposit
    } else {
        streamed_amount
    }
}

/*
Modifiers for the contract
*/
fn require_sender_or_recipient(stream: &Stream, user: &Address) {
    let sender = &(stream.sender);
    let recipient = &(stream.recipient);
    assert!(
        user == sender || user == recipient,
        "only sender or recipient can call this function"
    );
}

fn require_recipient(stream: &Stream, user: &Address) {
    let recipient = &(stream.recipient);
    assert!(
        user == recipient,
        "only recipient can receive the streamed amount"
    );
}

// Transfer tokens from the contract to the recipient
fn transfer(e: &Env, to: &Address, amount: &i128) {
    let token_contract_id = &get_token(e);
    let client = token::Client::new(e, token_contract_id);
    client.transfer(&e.current_contract_address(), to, amount);
}

fn set_stream(e: &Env, stream_id: &u32, stream: &Stream) {
    e.storage()
        .persistent()
        .set(&DataKey::Streams(stream_id.clone()), stream);
    e.storage().persistent().extend_ttl(
        &DataKey::Streams(stream_id.clone()),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_EXTEND_TTL_AMOUNT,
    );
}

fn set_stream_by_user(e: &Env, who: &Address, stream: &Stream) {
    // Get the array of streams for the user
    let mut streams = get_streams_by_user(&e, &who);
    streams.push_back(stream.clone());
    e.storage()
        .persistent()
        .set(&DataKey::UserStreams(who.clone()), &streams);
    e.storage().persistent().extend_ttl(
        &DataKey::UserStreams(who.clone()),
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_EXTEND_TTL_AMOUNT,
    );
}

fn set_next_stream_id(e: &Env, stream_id: &u32) {
    e.storage()
        .instance()
        .set(&DataKey::NextStreamId, &(stream_id + 1));
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_EXTEND_TTL_AMOUNT);
}

#[contract]
struct StreamToken;

#[contractimpl]
#[allow(clippy::needless_pass_by_value)]
impl StreamToken {
    pub fn initialize(e: Env, token: Address) {
        assert!(
            !e.storage().instance().has(&DataKey::NextStreamId),
            "already initialized"
        );

        let initial_stream_id: u32 = 1;
        e.storage()
            .instance()
            .set(&DataKey::NextStreamId, &initial_stream_id);
        e.storage().instance().set(&DataKey::Token, &token);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_EXTEND_TTL_AMOUNT);
    }

    pub fn get_stream(e: Env, stream_id: u32) -> Stream {
        get_stream(&e, &stream_id)
    }

    pub fn get_streams_by_user(e: Env, caller: Address) -> Vec<Stream> {
        get_streams_by_user(&e, &caller)
    }

    /// Returns the amount of tokens that have already been released to the recipient.
    /// Panics if the id does not point to a valid stream.
    /// @param stream_id The id of the stream
    /// @param who The address of the caller
    /// @return The amount of tokens that have already been released
    pub fn streamed_amount(e: Env, stream_id: u32) -> i128 {
        let stream = get_stream(&e, &stream_id);

        get_streamed_amount(&e, &stream)
    }

    // convert the above solidity function to soroban
    pub fn create_stream(
        e: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        token_address: Address,
        start_time: u64,
        stop_time: u64,
    ) -> u32 {
        sender.require_auth();
        assert!(
            recipient != e.current_contract_address(),
            "stream to the contract itself"
        );
        assert!(recipient != sender, "self streaming not allowed");
        assert!(amount > 0, "amount is zero or negative");
        assert!(
            start_time >= get_ledger_timestamp(&e),
            "start time before current ledger timestamp"
        );
        assert!(stop_time > start_time, "stop time before the start time");

        let stream_id = get_next_stream_id(&e);

        log!(&e, "create_stream_log");
        let token_contract_id = get_token(&e);
        let client = token::Client::new(&e, &token_contract_id);
        client.transfer(&sender, &e.current_contract_address(), &amount);

        let token_symbol = client.symbol();
        let token_decimals = client.decimals();

        let stream = Stream {
            id: stream_id.clone(),
            sender: sender.clone(),
            recipient: recipient.clone(),
            start_time: start_time,
            stop_time: stop_time,
            deposit: amount,
            is_cancelled: false,
            withdrawn: 0,
            token_address: token_address.clone(),
            token_symbol: token_symbol.clone(),
            token_decimals: token_decimals.clone(),
        };

        set_stream(&e, &stream_id, &stream);
        set_stream_by_user(&e, &sender, &stream);
        set_stream_by_user(&e, &recipient, &stream);
        set_next_stream_id(&e, &stream_id);

        events::create_stream(
            &e,
            stream_id.clone(),
            sender,
            recipient,
            amount,
            token_address,
            token_symbol,
            token_decimals,
            start_time,
            stop_time,
        );
        stream_id
    }

    pub fn withdraw_from_stream(
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

        let mut stream = get_stream(&e, &stream_id);
        require_sender_or_recipient(&stream, &caller);
        require_recipient(&stream, &recipient);

        let streamed_amount = get_streamed_amount(&e, &stream);
        assert!(
            amount >= streamed_amount,
            "amount exceeds the streamed amount"
        );

        stream.withdrawn = stream.withdrawn + amount;

        set_stream(&e, &stream_id, &stream);

        transfer(&e, &recipient, &amount);

        events::withdraw_from_stream(&e, recipient, stream_id, amount);
    }

    /// Cancels the stream and transfers the tokens back on a pro rata basis.
    /// Throws if the id does not point to a valid stream.
    /// Throws if the caller is not the sender or the recipient of the stream.
    /// Throws if there is a token transfer failure.
    /// @param stream_id The id of the stream to cancel.
    /// @return bool true=success, otherwise false.
    pub fn cancel_stream(e: Env, caller: Address, stream_id: u32) {
        caller.require_auth();
        let mut stream = get_stream(&e, &stream_id);
        require_sender_or_recipient(&stream, &caller);

        let streamed_amount = get_streamed_amount(&e, &stream);
        let recipient_balance = streamed_amount - stream.withdrawn;
        let sender_balance = stream.deposit - streamed_amount;

        stream.is_cancelled = true;

        set_stream(&e, &stream_id, &stream);

        if recipient_balance > 0 {
            transfer(&e, &stream.recipient, &recipient_balance);
        }
        if sender_balance > 0 {
            transfer(&e, &stream.sender, &sender_balance);
        }

        events::cancel_stream(
            &e,
            stream_id,
            stream.sender,
            stream.recipient,
            sender_balance,
            recipient_balance,
        );
    }
}
