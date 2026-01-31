//! Local state management for private token balances and commitments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{ClientError, Result};

/// Represents a single UTXO commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    /// The commitment hash
    pub commitment: String,
    /// The derived address (from secret)
    pub address: String,
    /// The balance held in this commitment
    pub balance: u128,
    /// The nonce used in this commitment
    pub nonce: u64,
    /// The secret key (stored encrypted in production)
    pub secret: String,
    /// Whether this commitment has been spent
    pub spent: bool,
}

/// Manages local private state
#[derive(Debug, Serialize, Deserialize)]
pub struct StateManager {
    /// Path to the state file
    #[serde(skip)]
    state_file: String,
    /// Map of commitment hash to commitment data
    commitments: HashMap<String, Commitment>,
    /// Known addresses and their secrets
    accounts: HashMap<String, String>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(state_file: &str) -> Result<Self> {
        let mut manager = Self {
            state_file: state_file.to_string(),
            commitments: HashMap::new(),
            accounts: HashMap::new(),
        };
        
        // Load existing state if file exists
        if Path::new(state_file).exists() {
            manager.load()?;
        }
        
        Ok(manager)
    }

    /// Load state from file
    fn load(&mut self) -> Result<()> {
        let data = fs::read_to_string(&self.state_file)?;
        let loaded: StateManager = serde_json::from_str(&data)?;
        self.commitments = loaded.commitments;
        self.accounts = loaded.accounts;
        Ok(())
    }

    /// Save state to file
    pub fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&self.state_file, data)?;
        Ok(())
    }

    /// Add a new account (address -> secret mapping)
    pub fn add_account(&mut self, address: String, secret: String) -> Result<()> {
        self.accounts.insert(address, secret);
        self.save()
    }

    /// Get secret for an address
    pub fn get_secret(&self, address: &str) -> Option<&String> {
        self.accounts.get(address)
    }

    /// Add a new commitment
    pub fn add_commitment(&mut self, commitment: Commitment) -> Result<()> {
        self.commitments.insert(commitment.commitment.clone(), commitment);
        self.save()
    }

    /// Get a commitment by hash
    pub fn get_commitment(&self, commitment_hash: &str) -> Option<&Commitment> {
        self.commitments.get(commitment_hash)
    }

    /// Get all unspent commitments for an address
    pub fn get_unspent_commitments(&self, address: &str) -> Vec<&Commitment> {
        self.commitments
            .values()
            .filter(|c| c.address == address && !c.spent)
            .collect()
    }

    /// Mark a commitment as spent
    pub fn mark_spent(&mut self, commitment_hash: &str) -> Result<()> {
        if let Some(commitment) = self.commitments.get_mut(commitment_hash) {
            commitment.spent = true;
            self.save()?;
            Ok(())
        } else {
            Err(ClientError::CommitmentNotFound(commitment_hash.to_string()))
        }
    }

    /// Get total balance for an address
    pub fn get_balance(&self, address: &str) -> u128 {
        self.get_unspent_commitments(address)
            .iter()
            .map(|c| c.balance)
            .sum()
    }

    /// Find a suitable commitment for spending
    pub fn find_spendable_commitment(&self, address: &str, amount: u128) -> Option<&Commitment> {
        self.get_unspent_commitments(address)
            .into_iter()
            .find(|c| c.balance >= amount)
    }

    /// Get all accounts
    pub fn list_accounts(&self) -> Vec<(&String, u128)> {
        self.accounts
            .keys()
            .map(|addr| (addr, self.get_balance(addr)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_and_get_commitment() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(temp_file.path().to_str().unwrap()).unwrap();

        let commitment = Commitment {
            commitment: "0x1234".to_string(),
            address: "0xabcd".to_string(),
            balance: 100,
            nonce: 1,
            secret: "0xsecret".to_string(),
            spent: false,
        };

        manager.add_commitment(commitment.clone()).unwrap();
        
        let retrieved = manager.get_commitment("0x1234").unwrap();
        assert_eq!(retrieved.balance, 100);
    }

    #[test]
    fn test_balance_calculation() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(temp_file.path().to_str().unwrap()).unwrap();

        let address = "0xabcd".to_string();
        
        manager.add_commitment(Commitment {
            commitment: "0x1".to_string(),
            address: address.clone(),
            balance: 100,
            nonce: 1,
            secret: "0xsecret".to_string(),
            spent: false,
        }).unwrap();

        manager.add_commitment(Commitment {
            commitment: "0x2".to_string(),
            address: address.clone(),
            balance: 50,
            nonce: 2,
            secret: "0xsecret".to_string(),
            spent: false,
        }).unwrap();

        assert_eq!(manager.get_balance(&address), 150);
    }

    #[test]
    fn test_mark_spent() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(temp_file.path().to_str().unwrap()).unwrap();

        let address = "0xabcd".to_string();
        
        manager.add_commitment(Commitment {
            commitment: "0x1".to_string(),
            address: address.clone(),
            balance: 100,
            nonce: 1,
            secret: "0xsecret".to_string(),
            spent: false,
        }).unwrap();

        assert_eq!(manager.get_balance(&address), 100);
        
        manager.mark_spent("0x1").unwrap();
        
        assert_eq!(manager.get_balance(&address), 0);
    }
}
