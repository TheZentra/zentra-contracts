use soroban_sdk::{Address, Env, String, Symbol};

pub(crate) fn create_stream(
    e: &Env,
    stream_id: u32,
    sender: Address,
    recipient: Address,
    deposit: i128,
    token_address: Address,
    token_symbol: String,
    token_decimals: u32,
    start_time: u64,
    stop_time: u64,
) {
    let topics = (
        Symbol::new(e, "create_stream"),
        sender,
        recipient,
        token_address,
    );
    e.events().publish(
        topics,
        (
            stream_id,
            deposit,
            token_symbol,
            token_decimals,
            start_time,
            stop_time,
        ),
    );
}

pub(crate) fn withdraw_from_stream(e: &Env, recipient: Address, stream_id: u32, amount: i128) {
    let topics = (Symbol::new(e, "withdraw_from_stream"), recipient);
    e.events().publish(topics, (stream_id, amount));
}

// cancel_stream
pub(crate) fn cancel_stream(
    e: &Env,
    stream_id: u32,
    sender: Address,
    recipient: Address,
    sender_balance: i128,
    recipient_balance: i128,
) {
    let topics = (Symbol::new(e, "cancel_stream"), sender, recipient);
    e.events()
        .publish(topics, (stream_id, sender_balance, recipient_balance));
}
