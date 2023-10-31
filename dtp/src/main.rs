#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// imports
use ethers::contract::Contract;
use ethers::{core::rand::thread_rng, prelude::*, signers::LocalWallet, types::U256, utils::hex};
use eyre::{bail, Result};
use std::{str::FromStr, sync::Arc};
use structopt::StructOpt;

use bindings::counter::COUNTER_ABI;

mod utils;
use utils::*;

/// TODO: able to parse like "1 ETH", "1000 Wei"
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
            dotenv::from_path("../contracts/.env").expect("Failed to get env variables");

            // get counter contract address
            let counter_address =
                std::env::var("COUNTER").expect("Failed to get COUNTER contract address");
            let counter_address = counter_address.parse::<Address>()?;

            // get load contract address
            let load_address = std::env::var("LOAD").expect("Failed to get LOAD contract address");
            let load_address = load_address.parse::<Address>()?;

            // connect to parsed Node RPC URL
            let provider = Provider::<Http>::try_from(opt.rpc_url)
                .expect("Failed to connect! Please provide a valid RPC URL");

            // Create a shared reference across threads (in each `.await` call). looks synchronous, but many async calls are made here.
            let client = Arc::new(provider.clone());

            // import private key into wallet (local) to get the address
            let private_key_bytes = hex::decode(&opt.initial_funded_account_private_key)?;
            let funder_wallet =
                LocalWallet::from_bytes(&private_key_bytes).expect("Wallet creation failed");
            let funder_address = funder_wallet.address();

            // get the funder balance (in Wei)
            let funder_balance_wei_initial = client
                .get_balance(funder_address, Some(client.get_block_number().await?.into()))
                .await?;
            let funder_balance_tssc_initial = wei_to_tssc_string(funder_balance_wei_initial);
            println!("\nFunder's initial balance: {} TSSC.\n=====", funder_balance_tssc_initial);

            // calculate the required balance (in Wei)
            let required_balance: U256 = U256::from(
                opt.funding_amount
                    .checked_mul(opt.num_accounts.into())
                    .expect("Error in subtraction of difference amount"),
            );

            // check for sufficient balance in funder's account
            assert!(
                funder_balance_wei_initial > required_balance,
                "{}",
                &format!(
                    "funder has insufficient balance by {:?}",
                    required_balance.checked_sub(funder_balance_wei_initial)
                )
            );

            // generate some new accounts and send funds to each of them
            let mut wallet_addresses = Vec::<Address>::new();
            let mut wallet_priv_keys = Vec::<String>::new();
            // generate multiple accounts based on the parsed number.
            for i in 0..opt.num_accounts {
                let mut rng: rand::rngs::ThreadRng = thread_rng();
                let wallet = LocalWallet::new(&mut rng);
                // println!("Successfully created new keypair.");
                let pub_key = wallet.address();
                println!("\nAddress[{i}]:     {:?}", pub_key);
                // TODO: [OPTIONAL] save the keypair into a local file or show in the output. Create a CLI flag like --to-console/--to-file
                let priv_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));
                println!("Private key[{i}]: {}", priv_key);
                wallet_addresses.push(pub_key);
                wallet_priv_keys.push(priv_key);

                // transfer funds
                // TODO: send as bundle outside the for-loop. create a array of signed tx in this loop.
                transfer_tssc(&provider, &funder_wallet, pub_key, U256::from(opt.funding_amount))
                    .await
                    .expect(&format!("error in sending fund to {}", pub_key));
            }

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
                        // TODO: Bundle transactions and send in the next available blocks
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
            let funder_balance_wei_final = client.get_balance(funder_address, None).await?;
            let funder_balance_tssc_final = wei_to_tssc_string(funder_balance_wei_final);
            println!("\n=====\nFunder's final balance: {} TSSC.", funder_balance_tssc_final);
            let spent_bal_tssc = wei_to_tssc_f64(
                funder_balance_wei_initial
                    .checked_sub(funder_balance_wei_final)
                    .expect("Invalid sub op."),
            );
            println!("Funder spent: {:.18} TSSC", spent_bal_tssc);
        }
        Err(e) => {
            bail!("{}", e);
        }
    }

    Ok(())
}
