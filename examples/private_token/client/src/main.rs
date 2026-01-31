//! Private Token CLI Client
//!
//! A command-line interface for privacy-preserving token operations.

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

use private_token_client::{
    crypto, ContractConfig, PrivateTokenContract, ProofGenerator, StateManager,
    prover::{MintInputs, TransferInputs},
    state::Commitment,
};

#[derive(Parser)]
#[command(name = "private-token")]
#[command(about = "Privacy-preserving token client using Noir ZK proofs")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to state file
    #[arg(long, default_value = "private_state.json")]
    state_file: String,

    /// Path to compiled circuits directory
    #[arg(long, default_value = "../circuits")]
    circuits_dir: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new account (secret key)
    NewAccount {
        /// Optional name/label for the account
        #[arg(long)]
        name: Option<String>,
    },

    /// List all accounts and balances
    Accounts,

    /// Check balance for an account
    Balance {
        /// Address (hex) or account name
        #[arg(long)]
        address: String,
    },

    /// Mint tokens to an address
    Mint {
        /// Recipient secret (hex)
        #[arg(long)]
        secret: String,

        /// Amount to mint
        #[arg(long)]
        amount: u128,
    },

    /// Transfer tokens privately
    Transfer {
        /// Sender secret (hex)
        #[arg(long)]
        from_secret: String,

        /// Recipient address (hex)
        #[arg(long)]
        to_address: String,

        /// Amount to transfer
        #[arg(long)]
        amount: u128,
    },

    /// Show commitment details
    ShowCommitment {
        /// Commitment hash (hex)
        #[arg(long)]
        commitment: String,
    },

    /// Export account info
    Export {
        /// Address to export
        #[arg(long)]
        address: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // Initialize state manager
    let mut state = StateManager::new(&cli.state_file)?;

    match cli.command {
        Commands::NewAccount { name } => {
            new_account(&mut state, name)?;
        }
        Commands::Accounts => {
            list_accounts(&state)?;
        }
        Commands::Balance { address } => {
            show_balance(&state, &address)?;
        }
        Commands::Mint { secret, amount } => {
            mint_tokens(&mut state, &cli.circuits_dir, &secret, amount).await?;
        }
        Commands::Transfer {
            from_secret,
            to_address,
            amount,
        } => {
            transfer_tokens(&mut state, &cli.circuits_dir, &from_secret, &to_address, amount)
                .await?;
        }
        Commands::ShowCommitment { commitment } => {
            show_commitment(&state, &commitment)?;
        }
        Commands::Export { address } => {
            export_account(&state, &address)?;
        }
    }

    Ok(())
}

fn new_account(state: &mut StateManager, name: Option<String>) -> anyhow::Result<()> {
    let secret = crypto::generate_secret();
    let address = crypto::derive_address(&secret);

    let secret_hex = crypto::bytes32_to_hex(&secret);
    let address_hex = crypto::bytes32_to_hex(&address);

    state.add_account(address_hex.clone(), secret_hex.clone())?;

    println!("✅ New account created!");
    println!("   Address: {}", address_hex);
    println!("   Secret:  {}", secret_hex);
    if let Some(n) = name {
        println!("   Name:    {}", n);
    }
    println!();
    println!("⚠️  IMPORTANT: Save your secret key securely!");
    println!("   Anyone with your secret can spend your tokens.");

    Ok(())
}

fn list_accounts(state: &StateManager) -> anyhow::Result<()> {
    let accounts = state.list_accounts();

    if accounts.is_empty() {
        println!("No accounts found. Create one with: private-token new-account");
        return Ok(());
    }

    println!("Accounts:");
    println!("{:-<60}", "");
    for (address, balance) in accounts {
        println!("Address: {}", address);
        println!("Balance: {} tokens", balance);
        println!("{:-<60}", "");
    }

    Ok(())
}

fn show_balance(state: &StateManager, address: &str) -> anyhow::Result<()> {
    let balance = state.get_balance(address);
    let unspent = state.get_unspent_commitments(address);

    println!("Address: {}", address);
    println!("Balance: {} tokens", balance);
    println!("Unspent UTXOs: {}", unspent.len());

    Ok(())
}

async fn mint_tokens(
    state: &mut StateManager,
    circuits_dir: &str,
    secret_hex: &str,
    amount: u128,
) -> anyhow::Result<()> {
    println!("🔒 Minting {} tokens privately...", amount);

    // Parse secret
    let secret = crypto::hex_to_bytes32(secret_hex)?;
    let address = crypto::derive_address(&secret);
    let address_hex = crypto::bytes32_to_hex(&address);

    // Generate nonce (use timestamp for simplicity)
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Compute commitment
    let output_commitment = crypto::compute_commitment(&address, amount, nonce);
    let output_commitment_hex = crypto::bytes32_to_hex(&output_commitment);

    // Create proof generator
    let transfer_path = format!("{}/private_transfer/target/private_transfer.json", circuits_dir);
    let mint_path = format!("{}/mint/target/mint.json", circuits_dir);
    
    // Check if circuits are compiled
    if !std::path::Path::new(&mint_path).exists() {
        println!("⚠️  Mint circuit not compiled. Run:");
        println!("   cd circuits/mint && nargo compile");
        println!();
        println!("Simulating mint for demo purposes...");
    }

    let mint_request_id = nonce;

    // In production, generate actual proof here
    // let prover = ProofGenerator::new(&transfer_path, &mint_path)?;
    // let proof = prover.generate_mint_proof(MintInputs { ... })?;
    // let contract = PrivateTokenContract::from_env()?;
    // let tx_hash = contract.mint(proof).await?;

    // For demo, just update local state
    let commitment = Commitment {
        commitment: output_commitment_hex.clone(),
        address: address_hex.clone(),
        balance: amount,
        nonce,
        secret: secret_hex.to_string(),
        spent: false,
    };
    state.add_commitment(commitment)?;

    println!("✅ Minted {} tokens", amount);
    println!("   Address: {}", address_hex);
    println!("   Commitment: {}", output_commitment_hex);
    println!();
    println!("Note: In production, this would submit a ZK proof to the blockchain.");

    Ok(())
}

async fn transfer_tokens(
    state: &mut StateManager,
    circuits_dir: &str,
    from_secret_hex: &str,
    to_address_hex: &str,
    amount: u128,
) -> anyhow::Result<()> {
    println!("🔒 Transferring {} tokens privately...", amount);

    // Parse inputs
    let sender_secret = crypto::hex_to_bytes32(from_secret_hex)?;
    let sender_address = crypto::derive_address(&sender_secret);
    let sender_address_hex = crypto::bytes32_to_hex(&sender_address);
    let recipient_address = crypto::hex_to_bytes32(to_address_hex)?;

    // Find spendable commitment
    let spendable = state
        .find_spendable_commitment(&sender_address_hex, amount)
        .ok_or_else(|| anyhow::anyhow!("Insufficient balance"))?
        .clone();

    let sender_balance = spendable.balance;
    let new_balance = sender_balance - amount;
    let new_nonce = spendable.nonce + 1;

    // Compute values
    let input_commitment = crypto::hex_to_bytes32(&spendable.commitment)?;
    let nullifier = crypto::compute_nullifier(&sender_secret, spendable.nonce);
    let output_commitment_sender = crypto::compute_commitment(&sender_address, new_balance, new_nonce);
    let output_commitment_recipient = crypto::compute_commitment(&recipient_address, amount, 0);

    // In production, generate proof and submit to blockchain
    // For demo, just update local state

    // Mark old commitment as spent
    state.mark_spent(&spendable.commitment)?;

    // Add new sender commitment if there's change
    if new_balance > 0 {
        state.add_commitment(Commitment {
            commitment: crypto::bytes32_to_hex(&output_commitment_sender),
            address: sender_address_hex.clone(),
            balance: new_balance,
            nonce: new_nonce,
            secret: from_secret_hex.to_string(),
            spent: false,
        })?;
    }

    // Add recipient commitment (they would need to import this)
    state.add_commitment(Commitment {
        commitment: crypto::bytes32_to_hex(&output_commitment_recipient),
        address: to_address_hex.to_string(),
        balance: amount,
        nonce: 0,
        secret: String::new(), // Recipient needs their own secret
        spent: false,
    })?;

    println!("✅ Transferred {} tokens", amount);
    println!("   From: {}", sender_address_hex);
    println!("   To: {}", to_address_hex);
    println!("   Nullifier: {}", crypto::bytes32_to_hex(&nullifier));
    println!();
    println!("Note: In production, this would submit a ZK proof to the blockchain.");

    Ok(())
}

fn show_commitment(state: &StateManager, commitment_hex: &str) -> anyhow::Result<()> {
    match state.get_commitment(commitment_hex) {
        Some(c) => {
            println!("Commitment Details:");
            println!("  Hash:    {}", c.commitment);
            println!("  Address: {}", c.address);
            println!("  Balance: {}", c.balance);
            println!("  Nonce:   {}", c.nonce);
            println!("  Spent:   {}", c.spent);
        }
        None => {
            println!("Commitment not found: {}", commitment_hex);
        }
    }
    Ok(())
}

fn export_account(state: &StateManager, address: &str) -> anyhow::Result<()> {
    match state.get_secret(address) {
        Some(secret) => {
            println!("Account Export:");
            println!("  Address: {}", address);
            println!("  Secret:  {}", secret);
            println!();
            println!("⚠️  Keep this information secure!");
        }
        None => {
            println!("Account not found: {}", address);
        }
    }
    Ok(())
}
