use crate::base::errors::Error;
// use crate::base::events::emit_autoshare_created; // Deprecated - TODO: migrate to #[contractevent]
use crate::base::types::{AutoShareDetails, GroupMember, PaymentHistory};
use soroban_sdk::{contracttype, token, Address, BytesN, Env, String, Vec};

#[contracttype]
pub enum DataKey {
    AutoShare(BytesN<32>),
    GroupMembers(BytesN<32>),
    AllGroups,
    Admin,
    SupportedTokens,
    UsageFee,
    UserPaymentHistory(Address),
    GroupPaymentHistory(BytesN<32>),
}

pub fn create_autoshare(
    env: Env,
    id: BytesN<32>,
    name: String,
    creator: Address,
    usage_count: u32,
    payment_token: Address,
) -> Result<(), Error> {
    creator.require_auth();

    let key = DataKey::AutoShare(id.clone());

    // Check if it already exists to prevent overwriting
    if env.storage().persistent().has(&key) {
        return Err(Error::AlreadyExists);
    }

    // Validate usage count
    if usage_count == 0 {
        return Err(Error::InvalidUsageCount);
    }

    // Verify token is supported
    if !is_token_supported(env.clone(), payment_token.clone()) {
        return Err(Error::UnsupportedToken);
    }

    // Calculate total cost
    let usage_fee = get_usage_fee(env.clone());
    let total_cost = (usage_count as i128) * (usage_fee as i128);

    // Transfer tokens from creator to contract
    let token_client = token::Client::new(&env, &payment_token);
    token_client.transfer(&creator, env.current_contract_address(), &total_cost);

    let details = AutoShareDetails {
        id: id.clone(),
        name,
        creator: creator.clone(),
        usage_count,
        total_usages_paid: usage_count,
    };

    // Store the details in persistent storage
    env.storage().persistent().set(&key, &details);

    // Add to all groups list
    let all_groups_key = DataKey::AllGroups;
    let mut all_groups: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(&env));
    all_groups.push_back(id.clone());
    env.storage().persistent().set(&all_groups_key, &all_groups);

    // Initialize empty members list
    let members_key = DataKey::GroupMembers(id.clone());
    let empty_members: Vec<GroupMember> = Vec::new(&env);
    env.storage().persistent().set(&members_key, &empty_members);

    // Record payment history
    record_payment(
        env.clone(),
        creator.clone(),
        id.clone(),
        usage_count,
        total_cost,
    );

    // TODO: Migrate to #[contractevent] macro instead of deprecated publish method
    // emit_autoshare_created(&env, id, creator);
    Ok(())
}

pub fn get_autoshare(env: Env, id: BytesN<32>) -> Result<AutoShareDetails, Error> {
    let key = DataKey::AutoShare(id);
    env.storage().persistent().get(&key).ok_or(Error::NotFound)
}

pub fn get_all_groups(env: Env) -> Vec<AutoShareDetails> {
    let all_groups_key = DataKey::AllGroups;
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(&env));

    let mut result: Vec<AutoShareDetails> = Vec::new(&env);
    for id in group_ids.iter() {
        if let Ok(details) = get_autoshare(env.clone(), id) {
            result.push_back(details);
        }
    }
    result
}

pub fn get_groups_by_creator(env: Env, creator: Address) -> Vec<AutoShareDetails> {
    let all_groups = get_all_groups(env.clone());
    let mut result: Vec<AutoShareDetails> = Vec::new(&env);

    for group in all_groups.iter() {
        if group.creator == creator {
            result.push_back(group);
        }
    }
    result
}

pub fn is_group_member(env: Env, id: BytesN<32>, address: Address) -> Result<bool, Error> {
    // First check if the group exists
    let group_key = DataKey::AutoShare(id.clone());
    if !env.storage().persistent().has(&group_key) {
        return Err(Error::NotFound);
    }

    let members_key = DataKey::GroupMembers(id);
    let members: Vec<GroupMember> = env
        .storage()
        .persistent()
        .get(&members_key)
        .unwrap_or(Vec::new(&env));

    for member in members.iter() {
        if member.address == address {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn get_group_members(env: Env, id: BytesN<32>) -> Result<Vec<GroupMember>, Error> {
    // First check if the group exists
    let group_key = DataKey::AutoShare(id.clone());
    if !env.storage().persistent().has(&group_key) {
        return Err(Error::NotFound);
    }

    let members_key = DataKey::GroupMembers(id);
    let members: Vec<GroupMember> = env
        .storage()
        .persistent()
        .get(&members_key)
        .unwrap_or(Vec::new(&env));

    Ok(members)
}

pub fn add_group_member(env: Env, id: BytesN<32>, address: Address) -> Result<(), Error> {
    // First check if the group exists
    let group_key = DataKey::AutoShare(id.clone());
    if !env.storage().persistent().has(&group_key) {
        return Err(Error::NotFound);
    }

    let members_key = DataKey::GroupMembers(id);
    let mut members: Vec<GroupMember> = env
        .storage()
        .persistent()
        .get(&members_key)
        .unwrap_or(Vec::new(&env));

    // Check if already a member
    for member in members.iter() {
        if member.address == address {
            return Err(Error::AlreadyExists);
        }
    }

    members.push_back(GroupMember { address });
    env.storage().persistent().set(&members_key, &members);
    Ok(())
}

// ============================================================================
// Admin Management
// ============================================================================

pub fn initialize_admin(env: Env, admin: Address) {
    admin.require_auth();
    let admin_key = DataKey::Admin;

    // Only set if not already initialized
    if !env.storage().persistent().has(&admin_key) {
        env.storage().persistent().set(&admin_key, &admin);

        // Initialize default usage fee (10 tokens per usage)
        let usage_fee_key = DataKey::UsageFee;
        env.storage().persistent().set(&usage_fee_key, &10u32);

        // Initialize empty supported tokens list
        let tokens_key = DataKey::SupportedTokens;
        let empty_tokens: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&tokens_key, &empty_tokens);
    }
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin_key = DataKey::Admin;
    let admin: Address = env
        .storage()
        .persistent()
        .get(&admin_key)
        .ok_or(Error::Unauthorized)?;

    if admin != *caller {
        return Err(Error::Unauthorized);
    }

    Ok(())
}

// ============================================================================
// Supported Tokens Management
// ============================================================================

pub fn add_supported_token(env: Env, token: Address, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let tokens_key = DataKey::SupportedTokens;
    let mut tokens: Vec<Address> = env
        .storage()
        .persistent()
        .get(&tokens_key)
        .unwrap_or(Vec::new(&env));

    // Check if token is already supported
    for existing_token in tokens.iter() {
        if existing_token == token {
            return Err(Error::AlreadyExists);
        }
    }

    tokens.push_back(token);
    env.storage().persistent().set(&tokens_key, &tokens);
    Ok(())
}

pub fn remove_supported_token(env: Env, token: Address, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let tokens_key = DataKey::SupportedTokens;
    let tokens: Vec<Address> = env
        .storage()
        .persistent()
        .get(&tokens_key)
        .unwrap_or(Vec::new(&env));

    let mut new_tokens: Vec<Address> = Vec::new(&env);
    let mut found = false;

    for existing_token in tokens.iter() {
        if existing_token != token {
            new_tokens.push_back(existing_token);
        } else {
            found = true;
        }
    }

    if !found {
        return Err(Error::NotFound);
    }

    env.storage().persistent().set(&tokens_key, &new_tokens);
    Ok(())
}

pub fn get_supported_tokens(env: Env) -> Vec<Address> {
    let tokens_key = DataKey::SupportedTokens;
    env.storage()
        .persistent()
        .get(&tokens_key)
        .unwrap_or(Vec::new(&env))
}

pub fn is_token_supported(env: Env, token: Address) -> bool {
    let tokens = get_supported_tokens(env);
    for supported_token in tokens.iter() {
        if supported_token == token {
            return true;
        }
    }
    false
}

// ============================================================================
// Payment Configuration
// ============================================================================

pub fn set_usage_fee(env: Env, fee: u32, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let fee_key = DataKey::UsageFee;
    env.storage().persistent().set(&fee_key, &fee);
    Ok(())
}

pub fn get_usage_fee(env: Env) -> u32 {
    let fee_key = DataKey::UsageFee;
    env.storage().persistent().get(&fee_key).unwrap_or(10u32)
}

// ============================================================================
// Subscription Management
// ============================================================================

pub fn topup_subscription(
    env: Env,
    id: BytesN<32>,
    additional_usages: u32,
    payment_token: Address,
    payer: Address,
) -> Result<(), Error> {
    payer.require_auth();

    // Validate usage count
    if additional_usages == 0 {
        return Err(Error::InvalidUsageCount);
    }

    // Verify group exists
    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    // Verify token is supported
    if !is_token_supported(env.clone(), payment_token.clone()) {
        return Err(Error::UnsupportedToken);
    }

    // Calculate cost
    let usage_fee = get_usage_fee(env.clone());
    let total_cost = (additional_usages as i128) * (usage_fee as i128);

    // Transfer tokens from payer to contract
    let token_client = token::Client::new(&env, &payment_token);
    token_client.transfer(&payer, env.current_contract_address(), &total_cost);

    // Update usage counts
    details.usage_count += additional_usages;
    details.total_usages_paid += additional_usages;

    // Save updated details
    env.storage().persistent().set(&key, &details);

    // Record payment history
    record_payment(env, payer, id, additional_usages, total_cost);

    Ok(())
}

// ============================================================================
// Payment History
// ============================================================================

fn record_payment(
    env: Env,
    user: Address,
    group_id: BytesN<32>,
    usages_purchased: u32,
    amount_paid: i128,
) {
    let timestamp = env.ledger().timestamp();

    let payment = PaymentHistory {
        user: user.clone(),
        group_id: group_id.clone(),
        usages_purchased,
        amount_paid,
        timestamp,
    };

    // Add to user's payment history
    let user_history_key = DataKey::UserPaymentHistory(user.clone());
    let mut user_history: Vec<PaymentHistory> = env
        .storage()
        .persistent()
        .get(&user_history_key)
        .unwrap_or(Vec::new(&env));
    user_history.push_back(payment.clone());
    env.storage()
        .persistent()
        .set(&user_history_key, &user_history);

    // Add to group's payment history
    let group_history_key = DataKey::GroupPaymentHistory(group_id);
    let mut group_history: Vec<PaymentHistory> = env
        .storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or(Vec::new(&env));
    group_history.push_back(payment);
    env.storage()
        .persistent()
        .set(&group_history_key, &group_history);
}

pub fn get_user_payment_history(env: Env, user: Address) -> Vec<PaymentHistory> {
    let user_history_key = DataKey::UserPaymentHistory(user);
    env.storage()
        .persistent()
        .get(&user_history_key)
        .unwrap_or(Vec::new(&env))
}

pub fn get_group_payment_history(env: Env, id: BytesN<32>) -> Vec<PaymentHistory> {
    let group_history_key = DataKey::GroupPaymentHistory(id);
    env.storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or(Vec::new(&env))
}

// ============================================================================
// Usage Tracking
// ============================================================================

pub fn get_remaining_usages(env: Env, id: BytesN<32>) -> Result<u32, Error> {
    let key = DataKey::AutoShare(id);
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    Ok(details.usage_count)
}

pub fn get_total_usages_paid(env: Env, id: BytesN<32>) -> Result<u32, Error> {
    let key = DataKey::AutoShare(id);
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    Ok(details.total_usages_paid)
}

pub fn reduce_usage(env: Env, id: BytesN<32>) -> Result<(), Error> {
    let key = DataKey::AutoShare(id);
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    if details.usage_count == 0 {
        return Err(Error::NoUsagesRemaining);
    }

    details.usage_count -= 1;
    env.storage().persistent().set(&key, &details);
    Ok(())
}
