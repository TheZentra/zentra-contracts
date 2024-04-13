use soroban_sdk::{contracttype, Address};

/// The governor settings for managing proposals
#[derive(Clone)]
#[contracttype]
pub struct StreamSettings {
    /// The address of the admin that can set protocol fees and perform other administrative functions.
    pub admin: Address,
    /// The fee that will apply if the token being streamed doesn't have a fee set.
    pub base_fee: i128,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct DirectStreamData {
    pub id: u32,
    pub sender: Address,
    pub recipient: Address,
    pub start_time: u64,
    pub cliff_time: u64,
    pub stop_time: u64,
    pub deposit: i128,
    pub withdrawn: i128,
    pub refunded: i128,
    pub is_cancellable: bool,
    pub is_cancelled: bool,
    pub is_depleted: bool,
    pub token_address: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum Status {
    Pending, // Stream has been created but not started yet
    Active, // Stream has started and is ongoing    
    Completed, // Stream has completed successfully. Recipient is yet to withdraw the funds
    Cancelled, // Stream has been cancelled. Funds have been refunded to the sender but recipient is yet to withdraw the funds
    Cliff, // Stream has started but has not reached the cliff time yet
    Depleted, // Stream has been completed or cancelled and all funds have been withdrawn
}
