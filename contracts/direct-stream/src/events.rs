use soroban_sdk::{Address, Env, Symbol};

pub struct StreamEvents {}

impl StreamEvents {
    /// Emitted when a stream is created
    ///
    /// - topics - `["stream_created", stream_id: u32, sender: Address]`
    /// - data - `[token_address: Address, recipient: Address, deposit: i128, start_time: u64, stop_time: u64]`
    pub fn stream_created(
        e: &Env,
        stream_id: u32,
        sender: Address,
        recipient: Address,
        deposit: i128,
        token_address: Address,
        start_time: u64,
        stop_time: u64,
    ) {
        let topics = (Symbol::new(&e, "stream_created"), stream_id, sender);
        e.events().publish(
            topics,
            (token_address, recipient, deposit, start_time, stop_time),
        );
    }

    /// Emitted when a proposal is canceled
    ///
    /// - topics - `["proposal_canceled", proposal_id: u32]`
    /// - data - ()
    pub(crate) fn stream_cancelled(e: &Env, stream_id: u32) {
        let topics = (Symbol::new(&e, "stream_cancelled"), stream_id);
        e.events().publish(topics, ());
    }

    /// Emitted when a proposal voting period is closed
    ///
    /// - topics - `["proposal_voting_closed", proposal_id: u32, status: u32, eta: u32]`
    /// - data - `final_votes: VoteCount`
    pub(crate) fn stream_withdrawn(e: &Env, recipient: Address, stream_id: u32, amount: i128) {
        let topics = (Symbol::new(e, "stream_withdrawn"), stream_id, recipient);
        e.events().publish(topics, amount);
    }
}
