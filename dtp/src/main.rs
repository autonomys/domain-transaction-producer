#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// imports
use ethers::{
    core::k256::ecdsa::SigningKey, core::rand::thread_rng, prelude::*, signers::LocalWallet,
    types::U256, utils::hex,
};
use eyre::{bail, Result};
use log::info;
use std::{str::FromStr, sync::Arc};
use structopt::StructOpt;

/// utils
mod utils;
use utils::*;

mod contracts;

/// TODO: able to parse like "1 ETH", "1000 Wei"
/// TODO: `transaction_type` can be made as optional in cases where just need to transfer
/// funds to newly created accounts share their account details with the set of users as pre-funded account.
#[derive(StructOpt, Debug)]
#[structopt(name = "dtp", about = "Domain Transaction Producer")]
/// CLI params
struct Cli {
    /// Number of accounts
    #[structopt(short = "a", long)]
    num_accounts: u32,

    /// Transaction type: light or heavy
    #[structopt(short = "t", long)]
    transaction_type: String,

    /// Number of blocks to run for
    #[structopt(short = "b", long)]
    num_blocks: Option<u32>,

    /// Initial funded account private key
    #[structopt(short = "k", long)]
    initial_funded_account_private_key: String,

    /// Funding amount
    #[structopt(short = "f", long)]
    funding_amount: u64,

    /// Subspace EVM (Nova) RPC node URL
    #[structopt(short = "r", long)]
    rpc_url: String,
}

#[derive(Debug)]
/// Transaction type
enum TransactionType {
    LIGHT,
    HEAVY,
}

/// Implement `FromStr` trait for TransactionType
impl FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LIGHT" => Ok(TransactionType::LIGHT),
            "HEAVY" => Ok(TransactionType::HEAVY),
            _ => Err(format!("\'{}\' is not a valid TransactionType", s)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Cli::from_args();

    // The new accounts are supposed to send transactions of type - "LIGHT"/"HEAVY"
    match opt.transaction_type.parse::<TransactionType>() {
        Ok(transaction_type) => {
            // get the .env
            dotenv::from_path("./dtp/.env").expect("Failed to get env variables");

            // init logger for debugging
            env_logger::init();

            // get the env variables
            let (counter_address, load_address, multicall_address, fund_contract_addr, chain_id) =
                get_env_vars().await?;

            // connect to parsed Node RPC URL
            let provider = Provider::<Http>::try_from(opt.rpc_url)
                .expect("Failed to connect! Please provide a valid RPC URL");

            // Create a shared reference across threads (in each `.await` call). looks synchronous, but many async calls are made here.
            let client = Arc::new(provider.clone());

            // Get funder wallet after importing funder private key and also check for required funder balance
            // in order to transfer the funds to the newly created accounts.
            let (funder_wallet, funder_address, funder_balance_wei_initial) =
                get_funder_wallet_and_check_required_balance(
                    client.clone(),
                    opt.initial_funded_account_private_key,
                    opt.funding_amount,
                    opt.num_accounts,
                )
                .await?;

            // generate new accounts and transfer TSSC
            let signers = gen_wallets_transfer_tssc(
                client.clone(),
                opt.num_accounts,
                funder_wallet,
                opt.funding_amount,
                fund_contract_addr,
                chain_id,
            )
            .await?;

            // handle light/heavy txs
            if let TransactionType::LIGHT = transaction_type {
                match opt.num_blocks {
                    Some(num_blocks) => {
                        // TODO: Bundle transactions and send in the {num_blocks} blocks based on different cases
                        // There are 3 cases:
                        // 1. num_accounts < num_blocks
                        // 2. num_accounts = num_blocks
                        // 3. num_accounts > num_blocks
                    }
                    None => {
                        // TODO: The progress bar should be used like ... blinking or something to indicate that the program is still running.
                        println!("Sending light transactions...");
                        // Approach-2: All new wallet accounts are sender for each call individually
                        // Say, all of them want to increment
                        multicall_light_txs_2(client.clone(), counter_address, signers, chain_id)
                            .await
                            .expect("Approach-2 failed.");

                        println!("Light transactions sent successfully.")
                    }
                }
            } else if let TransactionType::HEAVY = transaction_type {
                match opt.num_blocks {
                    Some(num_blocks) => {
                        // TODO: Bundle transactions and send in the {num_blocks} blocks based on different cases
                        // There are 3 cases:
                        // 1. num_accounts < num_blocks
                        // 2. num_accounts = num_blocks
                        // 3. num_accounts > num_blocks
                    }
                    None => {
                        // TODO: Bundle transactions and send in the next available blocks
                    }
                }
            }

            // Show the funder's final balance at the end
            show_funder_final_balance(client, funder_address, funder_balance_wei_initial).await?;
        }
        Err(e) => {
            bail!("{}", e);
        }
    }

    Ok(())
}
