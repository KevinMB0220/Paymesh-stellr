use soroban_sdk::{symbol_short, Address, BytesN, Env};

// This function publishes an event to the Stellar network
// TODO: Migrate to #[contractevent] macro instead of deprecated publish method
#[allow(deprecated)]
pub fn emit_autoshare_created(env: &Env, id: BytesN<32>, creator: Address) {
    // Topics help indexers filter for this specific event
    let topics = (symbol_short!("created"), creator);

    // Publish the event with the AutoShare ID as the data
    env.events().publish(topics, id);
}
