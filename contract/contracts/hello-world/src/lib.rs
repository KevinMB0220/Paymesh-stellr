#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

// 1. Declare the foundational modules (Requirement: Modular Structure)
pub mod base {
    pub mod errors;
    pub mod events;
    pub mod types;
}

pub mod interfaces {
    pub mod autoshare;
}

// 2. Declare the main logic file where the functions are implemented
mod autoshare_logic;

#[contract]
pub struct AutoShareContract;

#[contractimpl]
impl AutoShareContract {
    // ============================================================================
    // Admin Management
    // ============================================================================

    /// Initializes the contract admin. Can only be called once.
    pub fn initialize_admin(env: Env, admin: Address) {
        autoshare_logic::initialize_admin(env, admin);
    }

    // ============================================================================
    // AutoShare Group Management
    // ============================================================================

    /// Creates a new AutoShare plan with payment.
    /// Requirement: create_autoshare should store data, accept payment, and emit an event.
    pub fn create(
        env: Env,
        id: BytesN<32>,
        name: String,
        creator: Address,
        usage_count: u32,
        payment_token: Address,
    ) {
        autoshare_logic::create_autoshare(env, id, name, creator, usage_count, payment_token)
            .unwrap();
    }

    /// Retrieves an existing AutoShare plan.
    /// Requirement: get_autoshare should return the plan details.
    pub fn get(env: Env, id: BytesN<32>) -> base::types::AutoShareDetails {
        autoshare_logic::get_autoshare(env, id).unwrap()
    }

    /// Retrieves all AutoShare groups.
    pub fn get_all_groups(env: Env) -> Vec<base::types::AutoShareDetails> {
        autoshare_logic::get_all_groups(env)
    }

    /// Retrieves all AutoShare groups created by a specific address.
    pub fn get_groups_by_creator(env: Env, creator: Address) -> Vec<base::types::AutoShareDetails> {
        autoshare_logic::get_groups_by_creator(env, creator)
    }

    /// Checks if an address is a member of a specific group.
    pub fn is_group_member(env: Env, id: BytesN<32>, address: Address) -> bool {
        autoshare_logic::is_group_member(env, id, address).unwrap()
    }

    /// Retrieves all members of a specific group.
    pub fn get_group_members(env: Env, id: BytesN<32>) -> Vec<base::types::GroupMember> {
        autoshare_logic::get_group_members(env, id).unwrap()
    }

    /// Adds a member to a specific group.
    pub fn add_group_member(env: Env, id: BytesN<32>, address: Address) {
        autoshare_logic::add_group_member(env, id, address).unwrap();
    }

    // ============================================================================
    // Token Management
    // ============================================================================

    /// Adds a supported payment token (admin only).
    pub fn add_supported_token(env: Env, token: Address, admin: Address) {
        autoshare_logic::add_supported_token(env, token, admin).unwrap();
    }

    /// Removes a supported payment token (admin only).
    pub fn remove_supported_token(env: Env, token: Address, admin: Address) {
        autoshare_logic::remove_supported_token(env, token, admin).unwrap();
    }

    /// Returns all supported payment tokens.
    pub fn get_supported_tokens(env: Env) -> Vec<Address> {
        autoshare_logic::get_supported_tokens(env)
    }

    /// Checks if a token is supported.
    pub fn is_token_supported(env: Env, token: Address) -> bool {
        autoshare_logic::is_token_supported(env, token)
    }

    // ============================================================================
    // Payment Configuration
    // ============================================================================

    /// Sets the usage fee (admin only).
    pub fn set_usage_fee(env: Env, fee: u32, admin: Address) {
        autoshare_logic::set_usage_fee(env, fee, admin).unwrap();
    }

    /// Returns the current usage fee.
    pub fn get_usage_fee(env: Env) -> u32 {
        autoshare_logic::get_usage_fee(env)
    }

    // ============================================================================
    // Subscription Management
    // ============================================================================

    /// Tops up a group's subscription with additional usages.
    pub fn topup_subscription(
        env: Env,
        id: BytesN<32>,
        additional_usages: u32,
        payment_token: Address,
        payer: Address,
    ) {
        autoshare_logic::topup_subscription(env, id, additional_usages, payment_token, payer)
            .unwrap();
    }

    // ============================================================================
    // Payment History
    // ============================================================================

    /// Returns all payment history for a user.
    pub fn get_user_payment_history(env: Env, user: Address) -> Vec<base::types::PaymentHistory> {
        autoshare_logic::get_user_payment_history(env, user)
    }

    /// Returns all payment history for a group.
    pub fn get_group_payment_history(env: Env, id: BytesN<32>) -> Vec<base::types::PaymentHistory> {
        autoshare_logic::get_group_payment_history(env, id)
    }

    // ============================================================================
    // Usage Tracking
    // ============================================================================

    /// Returns the remaining usages for a group.
    pub fn get_remaining_usages(env: Env, id: BytesN<32>) -> u32 {
        autoshare_logic::get_remaining_usages(env, id).unwrap()
    }

    /// Returns the total usages paid for a group.
    pub fn get_total_usages_paid(env: Env, id: BytesN<32>) -> u32 {
        autoshare_logic::get_total_usages_paid(env, id).unwrap()
    }

    /// Reduces the usage count by 1 (dummy function for testing).
    pub fn reduce_usage(env: Env, id: BytesN<32>) {
        autoshare_logic::reduce_usage(env, id).unwrap();
    }
}

// 3. Link the tests (Requirement: Unit Tests)
#[cfg(test)]
#[path = "tests/autoshare_test.rs"]
mod autoshare_test; // Links the internal tests/autoshare_test.rs inside src
