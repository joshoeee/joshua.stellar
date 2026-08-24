#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, token};

#[contracttype]
pub enum DataKey {
    Advance(u64), // Advance ID -> Advance details
    Counter,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct InvoiceAdvance {
    pub farmer: Address,
    pub coop: Address,
    pub amount: i128,
    pub disbursed: bool,
}

#[contract]
pub struct AgriPayContract;

#[contractimpl]
impl AgriPayContract {
    /// Creates and immediately disburses an 80% USDC advance for verified crops delivered.
    pub fn create_and_disburse_advance(
        env: Env,
        usdc_token: Address,
        coop: Address,
        farmer: Address,
        crop_value: i128,
    ) -> u64 {
        // Enforce cooperative authorization
        coop.require_auth();

        // Calculate 80% advance amount
        let advance_amount = (crop_value * 80) / 100;

        // Transfer USDC from Cooperative to Farmer directly
        let client = token::Client::new(&env, &usdc_token);
        client.transfer(&coop, &farmer, &advance_amount);

        // Fetch current advance counter ID
        let mut id: u64 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0);
        id += 1;

        // Record the advance in contract storage
        let record = InvoiceAdvance {
            farmer: farmer.clone(),
            coop: coop.clone(),
            amount: advance_amount,
            disbursed: true,
        };
        
        env.storage().instance().set(&DataKey::Advance(id), &record);
        env.storage().instance().set(&DataKey::Counter, &id);

        id
    }

    /// Read function to verify state of an advance record
    pub fn get_advance(env: Env, id: u64) -> InvoiceAdvance {
        env.storage().instance().get(&DataKey::Advance(id)).unwrap()
    }
}