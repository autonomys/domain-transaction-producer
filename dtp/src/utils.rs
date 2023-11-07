use crate::contracts::{counter_get_number, counter_increment, load_set_array};
use bindings::fund::Fund;
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::*,
    signers::Wallet,
    utils::{format_units, hex},
};
use futures::future::join_all;
use log::info;
use std::sync::Arc;

/// Convert Wei to TSSC (in String)
pub(crate) fn wei_to_tssc_string(bal_wei: U256) -> String {
    let bal_tssc = format_units(bal_wei, "ether").unwrap();
    bal_tssc
}

/// Convert Wei to TSSC (in f64)
pub(crate) fn wei_to_tssc_f64(bal_wei: U256) -> f64 {
    let bal_tssc = bal_wei.as_usize() as f64 / 1e18;
    bal_tssc
}

/// calculate tx gas cost from `gas_price` & `gas_spent` for a tx
pub(crate) async fn get_gas_cost(
    provider: &Provider<Http>,
    tx_receipt: &TransactionReceipt,
) -> eyre::Result<f64> {
    let gas_price =
        provider.get_transaction(tx_receipt.transaction_hash).await?.unwrap().gas_price.unwrap();

    let gas_spent = provider.get_transaction(tx_receipt.transaction_hash).await?.unwrap().gas;

    let gas_cost_wei = gas_price.checked_mul(gas_spent).unwrap();
    let gas_cost_tssc = wei_to_tssc_f64(gas_cost_wei);

    Ok(gas_cost_tssc)
}

/// Handle future calls by batching method into a batch of max. chunk size.
/// Otherwise, without batching, it's failing when requested too many connections at once.
/// All new accounts are incrementing numbers (as considered this activity).
/// NOTE: There are many combination of running these txs by multiple signers.
/// But, for simplicity here, we have considered only `increment` function call
/// of `Counter` contract.
///
/// Here, instead of sending `calls` at once via `join_all(calls).await` as shown here:
/// ```rust
/// let mut calls = Vec::with_capacity(signers.len());
/// for signer in &signers {
///     // collect all contract setter calls into a vec of futures. So, not awaited on each future in this loop.
///     calls.push(counter_increment(client.clone(), counter_address, signer.to_owned()));
/// }
/// // async calls awaited all at once so as to try to put as many txs into a/few blocks than
/// // sending each tx into each block.
/// // Here, senders are different for each tx. So, not a problem of dependency on each other.
/// join_all(calls).await;
/// ```
///
/// Use `batch` to run each batch via `join_all(batch).await`. E.g. for 1000 connections,
/// there would be 10 batches of 100 calls/requests each. Now, each batch i.e. 100 requests
/// is sent at once, unlike all 1000 (total) calls sent at once as was done previously.
async fn handle_async_calls_in_batch_light(
    client: Arc<Provider<Http>>,
    counter_address: Address,
    signers: Vec<Wallet<SigningKey>>,
    chain_id: u64,
    max_batch_size: u16,
) -> eyre::Result<()> {
    // iteration in chunks of `MAX_BATCH_SIZE`
    for chunk in signers.chunks(max_batch_size.into()) {
        // create a batch vec for this chunk
        let mut batch = Vec::with_capacity(chunk.len());

        for signer in chunk {
            batch.push(counter_increment(
                client.clone(),
                counter_address,
                signer.to_owned(),
                chain_id,
            ));
        }

        // Send txs in a batch of `MAX_BATCH_SIZE`
        // If any of the futures in this batch returns an error, it will stop and return that error
        join_all(batch).await;
        // handle errors
    }

    Ok(())
}

/// Approach-2: All new wallet accounts are sender for each call
pub(crate) async fn multicall_light_txs_2(
    client: Arc<Provider<Http>>,
    counter_address: Address,
    signers: Vec<Wallet<SigningKey>>,
    chain_id: u64,
    max_batch_size: u16,
) -> eyre::Result<()> {
    // get the number value before calls
    let num_before = counter_get_number(client.clone(), counter_address)
        .await
        .expect("Unable to get Counter number before calls.");
    info!("Number stored in \'Counter\' before calls: {}\n", num_before);

    // Handle async calls in batches where each batch has `MAX_BATCH_SIZE` requests.
    handle_async_calls_in_batch_light(
        client.clone(),
        counter_address,
        signers.to_owned(),
        chain_id,
        max_batch_size,
    )
    .await?;

    // get the number value before calls
    let num_after = counter_get_number(client.clone(), counter_address)
        .await
        .expect("Unable to get Counter number after calls.");
    info!("Number stored in \'Counter\' after {} calls: {}\n", signers.clone().len(), num_after);

    Ok(())
}

/// Like `handle_async_calls_in_batch_light` but for HEAVY txs.
/// Considered `Load` contract's `setArray` method as HEAVY txs.
async fn handle_async_calls_in_batch_heavy(
    client: Arc<Provider<Http>>,
    load_address: Address,
    signers: Vec<Wallet<SigningKey>>,
    chain_id: u64,
    max_batch_size: u16,
    max_load_count_per_block: u16,
) -> eyre::Result<()> {
    // iteration in chunk of `MAX_BATCH_SIZE`
    for chunk in signers.chunks(max_batch_size.into()) {
        // create a batch vec for this chunk
        let mut batch = Vec::with_capacity(chunk.len());

        for signer in chunk {
            batch.push(load_set_array(
                client.clone(),
                load_address,
                signer.to_owned(),
                chain_id,
                max_load_count_per_block,
            ));
        }

        // Send txs in a batch of `MAX_BATCH_SIZE`
        // If any of the futures in this batch returns an error, it will stop and return that error
        join_all(batch).await;
    }

    Ok(())
}

/// As HEAVY transaction type, multicall particular function of Load contract
/// let's say `setArray` method successively done by each new accounts w/o
/// `num_txs` cli arg. Here, instead of sending `calls` at once via `join_all(calls).await`,
/// use `batch` to run each batch via `join_all(batch).await`. E.g. for 1000 connections,
/// there would be 10 batches of 100 calls/requests each. Now, each batch i.e. 100 requests
/// is sent at once, unlike all 1000 (total) calls sent at once as done similar for light txs.
pub(crate) async fn multicall_heavy_txs_2(
    client: Arc<Provider<Http>>,
    load_address: Address,
    signers: Vec<Wallet<SigningKey>>,
    chain_id: u64,
    max_batch_size: u16,
    max_load_count_per_block: u16,
) -> eyre::Result<()> {
    // Handle async calls in batches where each batch has `MAX_BATCH_SIZE` requests.
    handle_async_calls_in_batch_heavy(
        client.clone(),
        load_address,
        signers.to_owned(),
        chain_id,
        max_batch_size,
        max_load_count_per_block,
    )
    .await?;

    Ok(())
}

/// Get contract addresses from env variables from `.env` file
pub(crate) async fn get_env_vars() -> eyre::Result<(Address, Address, Address, Address, u16, u16)> {
    // get Counter contract address
    let counter_address =
        std::env::var("COUNTER").expect("Failed to get \'Counter\' contract address");
    let counter_address = counter_address.parse::<Address>()?;

    // get Load contract address
    let load_address = std::env::var("LOAD").expect("Failed to get \'LOAD\' contract address");
    let load_address = load_address.parse::<Address>()?;

    // get Multicall contract address
    let multicall_address =
        std::env::var("MULTICALL").expect("Failed to get \'Multicall\' contract address");
    let multicall_address = multicall_address.parse::<Address>()?;

    // get Fund contract address
    let fund_address = std::env::var("FUND").expect("Failed to get \'Fund\' contract address");
    let fund_address = fund_address.parse::<Address>()?;

    // get max batch size
    let max_batch_size = std::env::var("MAX_BATCH_SIZE").expect("Failed to get \'max batch size\'");
    let max_batch_size = max_batch_size.parse::<u16>()?;

    // get max load count per block
    let max_load_count_per_block = std::env::var("MAX_LOAD_COUNT_PER_BLOCK")
        .expect("Failed to get \'max load count per block\'");
    let max_load_count_per_block = max_load_count_per_block.parse::<u16>()?;

    Ok((
        counter_address,
        load_address,
        multicall_address,
        fund_address,
        max_batch_size,
        max_load_count_per_block,
    ))
}

/// Get funder wallet after importing funder private key and also check for required funder balance
/// in order to transfer the funds to the newly created accounts.
pub(crate) async fn get_funder_wallet_and_check_required_balance(
    client: Arc<Provider<Http>>,
    initial_funded_account_private_key: String,
    funding_amount: u64,
    num_accounts: u32,
) -> eyre::Result<(Wallet<SigningKey>, Address, U256)> {
    // import private key into wallet (local) to get the address
    let private_key_bytes = hex::decode(&initial_funded_account_private_key)?;
    let funder_wallet =
        LocalWallet::from_bytes(&private_key_bytes).expect("Wallet creation failed");
    let funder_address = funder_wallet.address();

    // get the funder balance (in Wei)
    let funder_balance_wei_initial = client.get_balance(funder_address, None).await?;
    let funder_balance_tssc_initial = wei_to_tssc_string(funder_balance_wei_initial);
    println!("\nFunder's initial balance: {} TSSC.\n=====", funder_balance_tssc_initial);

    // calculate the required balance (in Wei)
    let required_balance = U256::from(
        funding_amount
            .checked_mul(num_accounts.into())
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

    Ok((funder_wallet, funder_address, funder_balance_wei_initial))
}

/// Generates a specified number of wallets, funds them by calling a contract's `transferTsscToMany` method,
/// and returns the collection of generated wallets.
///
/// # Arguments
///
/// * `client` - An `Arc` wrapped `Provider` for HTTP requests.
/// * `num_accounts` - The number of wallets to generate.
/// * `funder_wallet` - The wallet instance used to fund the new wallets.
/// * `funding_amount` - The amount of funds to transfer to each wallet.
/// * `fund_contract_addr` - The smart contract address used for transferring funds.
/// * `chain_id` - The identifier of the specific Ethereum network chain being used.
///
/// # Returns
///
/// A result containing either a vector of wallets (`Vec<Wallet<SigningKey>>`) if successful, or an `eyre::Result` error
/// if the operation fails.
///
/// # Examples
///
/// ```
/// // Example usage (assuming async context and required variables are defined)
/// let wallets = gen_wallets_transfer_tssc(
///     client,
///     5,
///     funder_wallet,
///     1000,
///     fund_contract_addr,
///     1
/// ).await?;
/// ```
///
/// # Errors
///
/// This function will return an error if the contract's method call to transfer funds fails.
pub(crate) async fn gen_wallets_transfer_tssc(
    client: Arc<Provider<Http>>,
    num_accounts: u32,
    funder_wallet: Wallet<SigningKey>,
    funding_amount: u64,
    fund_contract_addr: Address,
    chain_id: u64,
) -> eyre::Result<Vec<Wallet<SigningKey>>> {
    // Use a thread-local random number generator
    let mut rng = rand::rngs::ThreadRng::default();

    // Generate wallets using the random number generator
    let wallets = (0..num_accounts).map(|_| LocalWallet::new(&mut rng)).collect::<Vec<_>>();

    // Extract the Ethereum addresses from the wallets
    let wallet_addresses = wallets
        .iter()
        .enumerate()
        .map(|(i, wallet)| {
            let address: H160 = wallet.address();
            println!("Address[{}]:     {:?}", i, address);
            address
        })
        .collect::<Vec<_>>();

    // TODO: [OPTIONAL] save the keypair into a local file or show in the output. Create a CLI flag like --to-console/--to-file
    // Extract and format the private keys of the wallets for logging purposes
    let wallet_priv_keys = wallets
        .iter()
        .enumerate()
        .map(|(i, wallet)| {
            let priv_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));
            println!("Private key[{}]: {}", i, priv_key);
            priv_key
        })
        .collect::<Vec<_>>();

    // Log the initiation of the bulk fund transfer operation
    println!("\nInitiating bulk transfer via the 'Fund' contract's 'transferTsscToMany' method...");

    // Perform the bulk transfer by invoking the contract's method
    transfer_tssc_bulk(
        client,
        &funder_wallet,
        wallet_addresses,
        U256::from(funding_amount),
        fund_contract_addr,
        chain_id,
    )
    .await?;

    // Return the wallets after funding
    Ok(wallets)
}

/// Transfer TSSC in bulk
pub(crate) async fn transfer_tssc_bulk(
    client: Arc<Provider<Http>>,
    from_wallet: &Wallet<SigningKey>,
    tos: Vec<Address>,
    funding_amount: U256,
    fund_contract_addr: Address,
    chain_id: u64,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware =
        SignerMiddleware::new(client.clone(), from_wallet.clone().with_chain_id(chain_id));

    // clone the client (if multiple use)
    let client_middleware = Arc::new(client_middleware);

    // get a contract
    let fund_contract = Fund::new(fund_contract_addr, client_middleware);

    // send a transaction with setter function
    let tx_receipt = fund_contract
        .transfer_tssc_to_many(tos.clone())
        .value(
            funding_amount
                .checked_mul(U256::from(tos.clone().len()))
                .expect("Error in multiplying fund amount w receivers len."),
        )
        .send()
        .await
        .expect("Failure in getting pending tx")
        .await?
        .expect("Failure in \'transferTsscToMany\' function of Fund contract");
    println!(
        "\n\'{}\' sent funds to newly created accounts, which incurred a gas fee of \'{} TSSC\', has a tx hash: \'{:?}\', indexed at #{} in block #{}.\n",
        tx_receipt.from,
        get_gas_cost(&client, &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}

/// Show the funder's final balance at the end
pub(crate) async fn show_funder_final_balance(
    client: Arc<Provider<Http>>,
    funder_address: Address,
    funder_balance_wei_initial: U256,
) -> eyre::Result<()> {
    let funder_balance_wei_final = client.get_balance(funder_address, None).await?;
    let funder_balance_tssc_final = wei_to_tssc_string(funder_balance_wei_final);
    println!("=====\nFunder's final balance: {} TSSC.", funder_balance_tssc_final);
    let spent_bal_tssc = wei_to_tssc_f64(
        funder_balance_wei_initial.checked_sub(funder_balance_wei_final).expect("Invalid sub op."),
    );
    println!("Funder spent: {:.18} TSSC", spent_bal_tssc);

    Ok(())
}
