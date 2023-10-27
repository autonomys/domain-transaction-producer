// imports
use ethers::{
    core::rand::thread_rng,
    prelude::*,
    signers::LocalWallet,
    types::transaction::{eip2718::TypedTransaction, eip2930::AccessList},
    utils::hex,
};
use eyre::{bail, Result};
use std::str::FromStr;
use structopt::StructOpt;

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
    /// DONE: Can be moved to `.env` file
    // #[structopt(short = "k", long)]
    // initial_funded_account_private_key: String,

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

/// Transfer TSSC function
/// TODO: [WIP] need to make production-ready
async fn transfer_tssc(provider: &Provider<Http>, private_key: &String) -> eyre::Result<()> {
    let private_key_bytes = hex::decode(private_key)?;

    // NOTE: Consider `from` as Signer (in ethers-ts typescript) unlike `to` that is considered as Address (H160).
    let from_wallet = LocalWallet::from_bytes(&private_key_bytes).expect("Wallet creation failed");
    let from = from_wallet.address();
    let to = "0xCa45D2A4993eF89BB881921fF6496C5CbDC78c23".parse::<Address>()?;

    // verify the balance of `from` > 1000
    let current_block_number: U64 = provider.get_block_number().await?;
    let wei_bal = provider
        .get_balance(from, Some(current_block_number.into()))
        .await?;
    assert!(
        wei_bal > U256::from(1000),
        "from\'s wei balance is insufficient"
    );

    let balance_before = provider.get_balance(from, None).await?;
    let nonce1 = provider.get_transaction_count(from, None).await?;

    // 1. create a tx
    let tx = TransactionRequest::new().to(to).value(1000).from(from);
    println!("\nTransfer ETH tx: {}", serde_json::to_string(&tx)?);

    // NOTE: broadcast it via the eth_sendTransaction API is disabled on pubic nodes (using infura, alchemy).
    // Only enabled on Anvil accounts (local network). Hence, use `eth_sendRawTransaction`.
    // Basically, sign it and then submit.
    // let tx = provider.send_transaction(tx, None).await?.await?;      // ERROR: `eth_sendTransaction` is not found on public nodes

    let chain_id = provider.get_chainid().await?;

    // 2. sign the tx
    let typed_tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
        from: Some(from),
        to: Some(to.into()),
        // ERROR: leading to replacement transaction underpriced in "Sepolia" network ❌, but runs fine in Subspace EVM network ✅
        gas: Some(U256::from(21000)),
        value: Some(U256::from(1000)),
        data: None,
        nonce: Some(nonce1),
        access_list: AccessList(vec![]),
        max_priority_fee_per_gas: None,
        max_fee_per_gas: Some(provider.get_gas_price().await?),
        chain_id: Some(chain_id.as_u64().into()),
    });
    println!("\nTyped tx: {:?}", typed_tx);
    println!("\nTyped tx hash: {:?}", typed_tx.sighash());

    let signature = from_wallet.sign_transaction(&typed_tx).await?;
    println!("\nSignature: {:?}", signature);

    // 3. serialize the signed tx to get the raw tx
    // RLP encoding has to be done as `Bytes` (ethers::types::Bytes) array
    let rlp_encoded_tx_bytes = typed_tx.rlp_signed(&signature);
    println!("\nRLP encoded tx bytes: {:?}", rlp_encoded_tx_bytes);

    // 4. send the raw transaction
    let tx_receipt = provider
        // `eth_sendRawTransaction` is run
        .send_raw_transaction(rlp_encoded_tx_bytes)
        .await
        .expect("Failure in raw tx [1]") // ERROR: tx cound not be decoded: couldn't decode RLP components: insufficient remaining input for short string", data: None
        .await
        .expect("Failure in raw tx [2]")
        .expect("Failure in getting tx receipt");
    println!(
        "Transaction sent with hash: {}",
        tx_receipt.transaction_hash
    );
    let nonce2 = provider.get_transaction_count(from, None).await?;

    assert!(nonce1 < nonce2);

    let balance_after = provider.get_balance(from, None).await?;
    assert!(balance_after < balance_before);

    println!("{} has balance before: {balance_before}", from);
    println!("{} has balance after: {balance_after}", from);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // load the .env file
    dotenv::from_path("./.env").expect("Failed in loading the .env file");

    // fetch the funder private key
    let funder_private_key = std::env::var("DEPLOYER_PRIVATE_KEY")
        .expect("Please check if \'DEPLOYER_PRIVATE_KEY\' is empty");

    let opt = Cli::from_args();

    match opt.transaction_type.parse::<TransactionType>() {
        Ok(transaction_type) => match opt.num_blocks {
            Some(num_blocks) => {
                // println!("Running {} transaction(s) of type {:?} for {} block(s) with {} account(s) and funding amount of {} wei using initial funded account with private key {} and Ethereum RPC node URL {}", opt.num_accounts, transaction_type, num_blocks, opt.num_accounts, opt.funding_amount, opt.initial_funded_account_private_key, opt.rpc_url);

                // get a provider connected to parsed RPC node URL
                let provider: Provider<Http> = Provider::<Http>::try_from(opt.rpc_url)
                    .expect("Failed to connect! Please provide a valid RPC URL");

                let mut wallet_addresses = Vec::<Address>::new();
                let mut wallet_priv_keys = Vec::<String>::new();
                // generate multiple accounts based on the parsed number.
                for _ in 0..opt.num_accounts {
                    let mut rng = thread_rng();
                    let wallet = LocalWallet::new(&mut rng);
                    // println!("Successfully created new keypair.");
                    // println!("Address:     {:?}", wallet.address());
                    // println!("Private key: 0x{}", hex::encode(wallet.signer().to_bytes()));
                    wallet_addresses.push(wallet.address());
                    wallet_priv_keys.push(format!("0x{}", hex::encode(wallet.signer().to_bytes())));
                }

                // TODO: transfer fund to each account from the set funder
                let private_key_bytes = hex::decode(&funder_private_key)?;
                let funder_wallet =
                    LocalWallet::from_bytes(&private_key_bytes).expect("Wallet creation failed");
                let funder_address = funder_wallet.address();

                let funder_bal_wei = provider
                    .get_balance(
                        funder_address,
                        Some(provider.get_block_number().await?.into()),
                    )
                    .await
                    .expect("Failure in getting funder's balance in wei");
                // if funder has non-zero balance
                if !funder_bal_wei.is_zero() {
                    // transfer funds to all the new accounts
                    for i in 0..opt.num_accounts {
                        transfer_tssc(&provider, &funder_private_key)
                            .await
                            .expect(&format!("error in sending fund to account[{}]", i));
                    }
                } else {
                    bail!("Funder has insufficient balance as {}.", funder_bal_wei)
                }
            }
            None => {
                // TODO: panic here may be.
                println!("Running {} transaction(s) of type {:?} indefinitely with {} account(s) and funding amount of {} wei using initial funded account with private key {} and Ethereum RPC node URL {}", 
                    opt.num_accounts, transaction_type, opt.num_accounts, opt.funding_amount, funder_private_key, opt.rpc_url);
            }
        },
        Err(e) => {
            println!("{}", e);
        }
    }

    Ok(())
}
