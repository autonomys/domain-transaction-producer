#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// imports
use ethers::{core::rand::thread_rng, prelude::*, signers::LocalWallet, types::U256, utils::hex};
use eyre::{bail, Result};
use std::str::FromStr;
use structopt::StructOpt;

mod utils;
use utils::{transfer_tssc, wei_to_tssc};

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
            _ => Err(format!("{} is not a valid TransactionType", s)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Cli::from_args();

    // connect to parsed Node RPC URL
    let provider: Provider<Http> = Provider::<Http>::try_from(opt.rpc_url)
        .expect("Failed to connect! Please provide a valid RPC URL");

    // import private key into wallet (local) to get the address
    let private_key_bytes = hex::decode(&opt.initial_funded_account_private_key)?;
    let funder_wallet =
        LocalWallet::from_bytes(&private_key_bytes).expect("Wallet creation failed");
    let funder_address = funder_wallet.address();

    // get the funder balance (in Wei)
    let funder_bal_wei_initial = provider
        .get_balance(
            funder_address,
            Some(provider.get_block_number().await?.into()),
        )
        .await?;
    let funder_balance_tssc_initial = wei_to_tssc(funder_bal_wei_initial);
    println!(
        "\nFunder's initial balance: {} TSSC.",
        funder_balance_tssc_initial
    );

    // calculate the required balance (in Wei)
    let required_balance: U256 = U256::from(
        opt.funding_amount
            .checked_mul(opt.num_accounts.into())
            .expect("Error in subtraction of difference amount"),
    );

    // check for sufficient balance in funder's account
    assert!(
        funder_bal_wei_initial > required_balance,
        "{}",
        &format!(
            "funder has insufficient balance by {:?}",
            required_balance.checked_sub(funder_bal_wei_initial)
        )
    );

    // 2. generate some new accounts and send funds to each of them
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

        // 3. transfer funds
        // TODO: make this as bundle (may be) outside the for-loop
        transfer_tssc(
            &provider,
            &funder_wallet,
            pub_key,
            U256::from(opt.funding_amount),
        )
        .await
        .expect(&format!("error in sending fund to {}", pub_key));
    }

    // 4. The new accounts are supposed to be sending transactions of type - "LIGHT"/"HEAVY"
    match opt.transaction_type.parse::<TransactionType>() {
        Ok(transaction_type) => match opt.num_blocks {
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
        },
        Err(e) => {
            bail!("Error: {}", e);
        }
    }

    // Show the funder's final balance at the end
    let funder_balance_wei_final = provider.get_balance(funder_address, None).await?;
    let funder_balance_tssc_final = wei_to_tssc(funder_balance_wei_final);
    println!(
        "\nFunder's final balance: {} TSSC.",
        funder_balance_tssc_final
    );

    Ok(())
}
