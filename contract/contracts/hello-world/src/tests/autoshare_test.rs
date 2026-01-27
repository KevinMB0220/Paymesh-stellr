use crate::{AutoShareContract, AutoShareContractClient};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    token, Address, BytesN, Env, IntoVal, String, Symbol,
};

// Helper function to create a mock token contract
fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &contract_address.address()),
        token::StellarAssetClient::new(env, &contract_address.address()),
    )
}

// Helper function to setup test environment with admin and tokens
fn setup_test_env() -> (
    Env,
    Address,
    AutoShareContractClient<'static>,
    Address,
    token::Client<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin
    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Create and register token
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client) = create_token_contract(&env, &token_admin);

    // Add token as supported
    client.add_supported_token(&token_client.address, &admin);

    (env, admin, client, token_client.address, token_client)
}

#[test]
fn test_create_and_get_success() {
    let (env, _admin, client, token_address, token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Platform Split");
    let usage_count = 100u32;

    // Mint tokens to creator
    let token_admin = Address::generate(&env);
    let (_, token_admin_client) = create_token_contract(&env, &token_admin);
    token_admin_client.mint(&creator, &1000000);

    client.create(&id, &name, &creator, &usage_count, &token_address);

    let result = client.get(&id);
    assert_eq!(result.name, name);
    assert_eq!(result.creator, creator);
    assert_eq!(result.usage_count, usage_count);
    assert_eq!(result.total_usages_paid, usage_count);
}

#[test]
#[should_panic]
fn test_duplicate_id_fails() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test");
    let creator = Address::generate(&env);

    client.create(&id, &name, &creator, &100u32, &token_address);
    client.create(&id, &name, &creator, &100u32, &token_address);
}

#[test]
#[should_panic]
fn test_get_non_existent_fails() {
    let (_env, _admin, client, _token_address, _token_client) = setup_test_env();

    let id = BytesN::from_array(&_env, &[9u8; 32]);
    client.get(&id);
}

#[test]
fn test_get_all_groups_empty() {
    let (_env, _admin, client, _token_address, _token_client) = setup_test_env();

    let groups = client.get_all_groups();
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_get_all_groups_multiple() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let id1 = BytesN::from_array(&env, &[1u8; 32]);
    let id2 = BytesN::from_array(&env, &[2u8; 32]);
    let id3 = BytesN::from_array(&env, &[3u8; 32]);
    let name1 = String::from_str(&env, "Group 1");
    let name2 = String::from_str(&env, "Group 2");
    let name3 = String::from_str(&env, "Group 3");

    client.create(&id1, &name1, &creator1, &100u32, &token_address);
    client.create(&id2, &name2, &creator2, &100u32, &token_address);
    client.create(&id3, &name3, &creator1, &100u32, &token_address);

    let groups = client.get_all_groups();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups.get(0).unwrap().id, id1);
    assert_eq!(groups.get(1).unwrap().id, id2);
    assert_eq!(groups.get(2).unwrap().id, id3);
}

#[test]
fn test_get_groups_by_creator_empty() {
    let (env, _admin, client, _token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let groups = client.get_groups_by_creator(&creator);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_get_groups_by_creator_multiple() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let id1 = BytesN::from_array(&env, &[1u8; 32]);
    let id2 = BytesN::from_array(&env, &[2u8; 32]);
    let id3 = BytesN::from_array(&env, &[3u8; 32]);
    let name1 = String::from_str(&env, "Group 1");
    let name2 = String::from_str(&env, "Group 2");
    let name3 = String::from_str(&env, "Group 3");

    client.create(&id1, &name1, &creator1, &100u32, &token_address);
    client.create(&id2, &name2, &creator2, &100u32, &token_address);
    client.create(&id3, &name3, &creator1, &100u32, &token_address);

    let groups = client.get_groups_by_creator(&creator1);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups.get(0).unwrap().id, id1);
    assert_eq!(groups.get(1).unwrap().id, id3);

    let groups2 = client.get_groups_by_creator(&creator2);
    assert_eq!(groups2.len(), 1);
    assert_eq!(groups2.get(0).unwrap().id, id2);
}

#[test]
fn test_is_group_member_false() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator, &100u32, &token_address);

    let is_member = client.is_group_member(&id, &member);
    assert!(!is_member);
}

#[test]
fn test_is_group_member_true() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator, &100u32, &token_address);
    client.add_group_member(&id, &member);

    let is_member = client.is_group_member(&id, &member);
    assert!(is_member);
}

#[test]
#[should_panic]
fn test_is_group_member_non_existent_group() {
    let (env, _admin, client, _token_address, _token_client) = setup_test_env();

    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[99u8; 32]);

    client.is_group_member(&id, &member);
}

#[test]
fn test_get_group_members_empty() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator, &100u32, &token_address);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 0);
}

#[test]
fn test_get_group_members_multiple() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator, &100u32, &token_address);
    client.add_group_member(&id, &member1);
    client.add_group_member(&id, &member2);
    client.add_group_member(&id, &member3);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 3);
    assert_eq!(members.get(0).unwrap().address, member1);
    assert_eq!(members.get(1).unwrap().address, member2);
    assert_eq!(members.get(2).unwrap().address, member3);
}

#[test]
#[should_panic]
fn test_get_group_members_non_existent_group() {
    let (env, _admin, client, _token_address, _token_client) = setup_test_env();

    let id = BytesN::from_array(&env, &[99u8; 32]);
    client.get_group_members(&id);
}

#[test]
#[should_panic]
fn test_add_member_to_non_existent_group() {
    let (env, _admin, client, _token_address, _token_client) = setup_test_env();

    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[99u8; 32]);
    client.add_group_member(&id, &member);
}

#[test]
#[should_panic]
fn test_add_duplicate_member() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator, &100u32, &token_address);
    client.add_group_member(&id, &member);
    client.add_group_member(&id, &member);
}

// ============================================================================
// Payment System Tests
// ============================================================================

#[test]
fn test_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Check default usage fee is set
    let fee = client.get_usage_fee();
    assert_eq!(fee, 10u32);

    // Check supported tokens list is empty
    let tokens = client.get_supported_tokens();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_add_and_get_supported_tokens() {
    let (env, admin, client, token_address, _token_client) = setup_test_env();

    let token2 = Address::generate(&env);
    client.add_supported_token(&token2, &admin);

    let tokens = client.get_supported_tokens();
    assert_eq!(tokens.len(), 2);
    assert!(client.is_token_supported(&token_address));
    assert!(client.is_token_supported(&token2));
}

#[test]
#[should_panic]
fn test_add_duplicate_token_fails() {
    let (env, admin, client, token_address, _token_client) = setup_test_env();

    client.add_supported_token(&token_address, &admin);
}

#[test]
fn test_remove_supported_token() {
    let (env, admin, client, token_address, _token_client) = setup_test_env();

    client.remove_supported_token(&token_address, &admin);

    let tokens = client.get_supported_tokens();
    assert_eq!(tokens.len(), 0);
    assert!(!client.is_token_supported(&token_address));
}

#[test]
#[should_panic]
fn test_remove_non_existent_token_fails() {
    let (env, admin, client, _token_address, _token_client) = setup_test_env();

    let non_existent_token = Address::generate(&env);
    client.remove_supported_token(&non_existent_token, &admin);
}

#[test]
fn test_set_and_get_usage_fee() {
    let (env, admin, client, _token_address, _token_client) = setup_test_env();

    let new_fee = 25u32;
    client.set_usage_fee(&new_fee, &admin);

    let fee = client.get_usage_fee();
    assert_eq!(fee, new_fee);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_set_usage_fee() {
    let (env, _admin, client, _token_address, _token_client) = setup_test_env();

    let non_admin = Address::generate(&env);
    let new_fee = 25u32;
    client.set_usage_fee(&new_fee, &non_admin);
}

#[test]
fn test_create_group_with_payment() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let usage_count = 50u32;

    client.create(&id, &name, &creator, &usage_count, &token_address);

    let details = client.get(&id);
    assert_eq!(details.usage_count, usage_count);
    assert_eq!(details.total_usages_paid, usage_count);
}

#[test]
#[should_panic]
fn test_create_group_with_unsupported_token_fails() {
    let (env, _admin, client, _token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let unsupported_token = Address::generate(&env);

    client.create(&id, &name, &creator, &50u32, &unsupported_token);
}

#[test]
#[should_panic]
fn test_create_group_with_zero_usages_fails() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Invalid Group");

    client.create(&id, &name, &creator, &0u32, &token_address);
}

#[test]
fn test_usage_fee_calculation() {
    let (env, admin, client, token_address, _token_client) = setup_test_env();

    // Set custom usage fee
    let usage_fee = 20u32;
    client.set_usage_fee(&usage_fee, &admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let usage_count = 100u32;

    client.create(&id, &name, &creator, &usage_count, &token_address);

    let details = client.get(&id);
    assert_eq!(details.usage_count, usage_count);
}

#[test]
fn test_topup_subscription() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let initial_usage = 50u32;

    client.create(&id, &name, &creator, &initial_usage, &token_address);

    // Top up with additional usages
    let additional_usages = 30u32;
    let payer = Address::generate(&env);
    client.topup_subscription(&id, &additional_usages, &token_address, &payer);

    let details = client.get(&id);
    assert_eq!(details.usage_count, initial_usage + additional_usages);
    assert_eq!(
        details.total_usages_paid,
        initial_usage + additional_usages
    );
}

#[test]
#[should_panic]
fn test_topup_with_zero_usages_fails() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");

    client.create(&id, &name, &creator, &50u32, &token_address);

    let payer = Address::generate(&env);
    client.topup_subscription(&id, &0u32, &token_address, &payer);
}

#[test]
#[should_panic]
fn test_topup_non_existent_group_fails() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let non_existent_id = BytesN::from_array(&env, &[99u8; 32]);
    let payer = Address::generate(&env);
    client.topup_subscription(&non_existent_id, &10u32, &token_address, &payer);
}

#[test]
#[should_panic]
fn test_topup_with_unsupported_token_fails() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");

    client.create(&id, &name, &creator, &50u32, &token_address);

    let unsupported_token = Address::generate(&env);
    let payer = Address::generate(&env);
    client.topup_subscription(&id, &10u32, &unsupported_token, &payer);
}

#[test]
fn test_payment_history_recording() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let usage_count = 50u32;

    client.create(&id, &name, &creator, &usage_count, &token_address);

    // Check user payment history
    let user_history = client.get_user_payment_history(&creator);
    assert_eq!(user_history.len(), 1);
    assert_eq!(user_history.get(0).unwrap().user, creator);
    assert_eq!(user_history.get(0).unwrap().usages_purchased, usage_count);

    // Check group payment history
    let group_history = client.get_group_payment_history(&id);
    assert_eq!(group_history.len(), 1);
    assert_eq!(group_history.get(0).unwrap().group_id, id);
}

#[test]
fn test_payment_history_multiple_payments() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");

    client.create(&id, &name, &creator, &50u32, &token_address);

    let payer = Address::generate(&env);
    client.topup_subscription(&id, &30u32, &token_address, &payer);

    // Check group payment history has both transactions
    let group_history = client.get_group_payment_history(&id);
    assert_eq!(group_history.len(), 2);

    // Check payer history
    let payer_history = client.get_user_payment_history(&payer);
    assert_eq!(payer_history.len(), 1);
    assert_eq!(payer_history.get(0).unwrap().usages_purchased, 30u32);
}

#[test]
fn test_get_remaining_usages() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let usage_count = 100u32;

    client.create(&id, &name, &creator, &usage_count, &token_address);

    let remaining = client.get_remaining_usages(&id);
    assert_eq!(remaining, usage_count);
}

#[test]
fn test_get_total_usages_paid() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let usage_count = 100u32;

    client.create(&id, &name, &creator, &usage_count, &token_address);

    let total = client.get_total_usages_paid(&id);
    assert_eq!(total, usage_count);
}

#[test]
fn test_reduce_usage() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");
    let usage_count = 10u32;

    client.create(&id, &name, &creator, &usage_count, &token_address);

    // Reduce usage
    client.reduce_usage(&id);

    let remaining = client.get_remaining_usages(&id);
    assert_eq!(remaining, usage_count - 1);

    // Total usages paid should remain the same
    let total = client.get_total_usages_paid(&id);
    assert_eq!(total, usage_count);
}

#[test]
#[should_panic]
fn test_reduce_usage_when_zero_fails() {
    let (env, _admin, client, token_address, _token_client) = setup_test_env();

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Paid Group");

    client.create(&id, &name, &creator, &1u32, &token_address);

    // Reduce to zero
    client.reduce_usage(&id);

    // This should panic
    client.reduce_usage(&id);
}

#[test]
#[should_panic]
fn test_reduce_usage_non_existent_group_fails() {
    let (_env, _admin, client, _token_address, _token_client) = setup_test_env();

    let non_existent_id = BytesN::from_array(&_env, &[99u8; 32]);
    client.reduce_usage(&non_existent_id);
}
