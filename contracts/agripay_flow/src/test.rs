#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal};
    use soroban_sdk::token::{Client as TokenClient, AdminClient as TokenAdminClient};

    fn setup_test() -> (Env, AgriPayContractClient<'static>, Address, Address, Address, TokenClient<'static>) {
        let env = Env::default();
        env.mock_all_signatures();

        let contract_id = env.register_contract(None, AgriPayContract);
        let client = AgriPayContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let coop = Address::generate(&env);
        let farmer = Address::generate(&env);

        let token_id = env.register_stellar_asset_contract(admin.clone());
        let token_admin = TokenAdminClient::new(&env, &token_id);
        let token_client = TokenClient::new(&env, &token_id);

        // Mint USDC tokens to Coop
        token_admin.mint(&coop, &1000);

        (env, client, coop, farmer, token_id, token_client)
    }

    #[test]
    fn test_1_happy_path_disburse_advance() {
        let (env, client, coop, farmer, token_id, token_client) = setup_test();

        let advance_id = client.create_and_disburse_advance(&token_id, &coop, &farmer, &100);

        assert_eq!(advance_id, 1);
        assert_eq!(token_client.balance(&farmer), 80);
        assert_eq!(token_client.balance(&coop), 920);
    }

    #[test]
    #[should_panic]
    fn test_2_edge_case_insufficient_coop_balance() {
        let (env, client, coop, farmer, token_id, _) = setup_test();

        // Attempting to advance on a $10,000 delivery when Coop only has $1,000 balance
        client.create_and_disburse_advance(&token_id, &coop, &farmer, &10000);
    }

    #[test]
    fn test_3_state_verification() {
        let (env, client, coop, farmer, token_id, _) = setup_test();

        let advance_id = client.create_and_disburse_advance(&token_id, &coop, &farmer, &500);
        let record = client.get_advance(&advance_id);

        assert_eq!(record.farmer, farmer);
        assert_eq!(record.coop, coop);
        assert_eq!(record.amount, 400); // 80% of 500
        assert_eq!(record.disbursed, true);
    }

    #[test]
    fn test_4_multiple_advances_increment_ids() {
        let (env, client, coop, farmer, token_id, _) = setup_test();

        let id1 = client.create_and_disburse_advance(&token_id, &coop, &farmer, &100);
        let id2 = client.create_and_disburse_advance(&token_id, &coop, &farmer, &200);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_5_zero_crop_value_yields_zero_advance() {
        let (env, client, coop, farmer, token_id, token_client) = setup_test();

        let advance_id = client.create_and_disburse_advance(&token_id, &coop, &farmer, &0);
        let record = client.get_advance(&advance_id);

        assert_eq!(record.amount, 0);
        assert_eq!(token_client.balance(&farmer), 0);
    }
}