use bindings::{counter::Counter, fund::Fund, load::Load};
use ethers::{
    core::{k256::ecdsa::SigningKey, rand::thread_rng},
    prelude::*,
    signers::Wallet,
    types::transaction::{eip2718::TypedTransaction, eip2930::AccessList},
    utils::{format_units, hex},
};
use futures::future::join_all;
use std::sync::Arc;

// max. no. of txs that can be sent in a batch unlike
// sending all txs at once which is failing due
// to too many connections at once.
const MAX_BATCH_SIZE: u16 = 100;

// max. value of `count` in `setArray` method of Load contract
// which is allowed to be added in a block. So, with this value
// set, we get the gas cost of (59.98 M) ~60 M per block. Foundry
// tests done with numerous value in `Load.t.sol` file.
const MAX_LOAD_COUNT_PER_BLOCK: u16 = 2650;

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
async fn get_gas_cost(
    provider: &Provider<Http>,
    tx_receipt: &TransactionReceipt,
) -> eyre::Result<f64> {
    let gas_price =
        provider.get_transaction(tx_receipt.transaction_hash).await?.unwrap().gas_price.unwrap();
    // println!("gas price: {} Wei", gas_price);

    let gas_spent = provider.get_transaction(tx_receipt.transaction_hash).await?.unwrap().gas;
    // println!("gas spent: {}", gas_spent);

    let gas_cost_wei = gas_price.checked_mul(gas_spent).unwrap();
    let gas_cost_tssc = wei_to_tssc_f64(gas_cost_wei);
    // println!("gas cost: {} TSSC", gas_cost_tssc);

    Ok(gas_cost_tssc)
}

/// get Counter number
/// NOTE: No signer needed as it is gasless call.
pub(crate) async fn counter_get_number(
    client: Arc<Provider<Http>>,
    counter_address: Address,
) -> eyre::Result<U256> {
    let counter = Counter::new(counter_address, client);

    let num = counter.number().call().await?;

    Ok(num)
}
/// set Counter number
/// NOTE: signer needed as it incurs gas fees.
#[allow(dead_code)]
pub(crate) async fn counter_set_number(
    client: Arc<Provider<Http>>,
    counter_address: Address,
    signer: Wallet<SigningKey>,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware = SignerMiddleware::new(
        client.clone(),
        signer.with_chain_id(client.get_chainid().await?.as_u64()),
    );

    // clone the client (if multiple use)
    let client_middleware = Arc::new(client_middleware);

    // get a contract
    let counter = Counter::new(counter_address, client_middleware);

    // send a transaction with setter function
    let tx_receipt = counter
        .set_number(U256::from(42))
        .send()
        .await?
        .await?
        .expect("Failure in \'set_number\' of Counter contract");
    println!(
        "\n\'{}\' set number as \'42\', which incurred a gas fee of \'{} TSSC\' has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        tx_receipt.from,
        get_gas_cost(&client.clone(), &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}

/// increment Counter number
/// NOTE: signer needed as it incurs gas fees.
pub(crate) async fn counter_increment(
    client: Arc<Provider<Http>>,
    counter_address: Address,
    signer: Wallet<SigningKey>,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware = SignerMiddleware::new(
        client.clone(),
        signer.with_chain_id(client.get_chainid().await?.as_u64()),
    );

    // clone the client (if multiple use)
    let client_middleware = Arc::new(client_middleware);

    // get a contract
    let counter = Counter::new(counter_address, client_middleware);

    // send a transaction with setter function
    let tx_receipt = counter
        .increment()
        .send()
        .await
        .expect("Failure in getting pending tx")
        .await?
        .expect("Failure in \'increment\' of Counter contract");
    println!(
        "\n\'{}\' increment number, which incurred a gas fee of \'{} TSSC\', has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        tx_receipt.from,
        get_gas_cost(&client.clone(), &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}

/// Approach-1: Only one sender account
/// NOTE: LIGHT txs sent as a batch could be signed by single/multiple signer,
/// but no need when calling any storage value.
/// As LIGHT transaction type, multicall particular function
/// let's say `increment` successively done by each new accounts w/o
/// `num_block` cli arg.
#[allow(dead_code)]
pub(crate) async fn multicall_light_txs_1(
    client: Arc<Provider<Http>>,
    multicall_address: Address,
    counter_address: Address,
    signers: Vec<Wallet<SigningKey>>,
) -> eyre::Result<()> {
    // initiate the Multicall instance and add calls one by one in builder style
    let mut multicall: Multicall<Provider<Http>> =
        Multicall::<Provider<Http>>::new(client.clone(), Some(multicall_address)).await.unwrap();

    // CLEANUP: remove later
    // let mut client_middlewares: Vec<SignerMiddleware<Arc<Provider<Http>>, Wallet<SigningKey>>> =
    // Vec::new();

    // create a middleware client with signature for each signer
    for _ in signers {
        // TODO: how to add signer middleware for signer to sign each call & then add to `multicall`
        // let client_middleware = SignerMiddleware::new(
        //     client.clone(),
        //     signer.with_chain_id(client.get_chainid().await?.as_u64()),
        // );

        // clone the client (if multiple use)
        // let client_middleware = Arc::new(client_middleware);

        // CLEANUP: remove later
        // client_middlewares.push(client);

        // get a contract
        // let counter = Counter::new(counter_address, client_middleware);  // for signer
        let counter = Counter::new(counter_address, client.clone());

        // note that these [`FunctionCall`]s are futures, and need to be `.await`ed to resolve.
        // But we will let `Multicall` to take care of that for us
        let counter_inc_call = counter.increment();
        // let counter_inc_call =
        // counter.method::<_, H256>("increment", false).expect("decoding error");

        // add call to the multicall
        multicall.add_call(counter_inc_call, false);
    }

    // `await`ing the `send` method waits for the transaction to be broadcast, which also
    // returns the transaction hash
    // FIXME: here, the multicall fails due to this error at `.expect("error in....`
    // ```
    // thread 'main' panicked at 'error in sending tx:
    // ContractError(MiddlewareError { e: JsonRpcClientError(JsonRpcError(JsonRpcError { code: -32603, message: "execution fatal: Module(ModuleError { index: 81, error: [0, 0, 0, 0], message: None })", data: None })) })', dtp/src/utils.rs:258:32
    // ```
    let tx_receipt =
        multicall.send().await.expect("error in sending tx").await.expect("tx dropped").unwrap();
    println!(
        "\'{}\' sent batch txs via \'multicall\', which incurred a gas fee of \'{} TSSC\', has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        tx_receipt.from,
        get_gas_cost(&client.clone(), &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
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
/// let mut calls = Vec::new();
/// for signer in &signers {
///     // collect all contract setter calls into a vec of futures. So, not awaited on each future in this loop.
///     calls.push(counter_increment(client.to_owned(), counter_address, signer.to_owned()));
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
) -> eyre::Result<()> {
    // iteration in chunk of `MAX_BATCH_SIZE`
    for chunk in signers.chunks(MAX_BATCH_SIZE.into()) {
        // create a batch vec for this chunk
        let mut batch = Vec::new();

        for signer in chunk {
            batch.push(counter_increment(client.to_owned(), counter_address, signer.to_owned()));
        }

        // send txs in batch of 100 (set now, can be adjusted later)
        join_all(batch).await;
    }

    Ok(())
}

/// Approach-2: All new wallet accounts are sender for each call
pub(crate) async fn multicall_light_txs_2(
    client: Arc<Provider<Http>>,
    counter_address: Address,
    signers: Vec<Wallet<SigningKey>>,
) -> eyre::Result<()> {
    // get the number value before calls
    let num_before = counter_get_number(client.clone(), counter_address)
        .await
        .expect("Unable to get Counter number before calls.");
    println!("\nNumber stored in \'Counter\' before calls: {}", num_before);

    // Handle async calls in batches where each batch has `MAX_BATCH_SIZE` requests.
    handle_async_calls_in_batch_light(client.to_owned(), counter_address, signers.to_owned())
        .await?;

    // get the number value before calls
    let num_after = counter_get_number(client.clone(), counter_address)
        .await
        .expect("Unable to get Counter number after calls.");
    println!("\nNumber stored in \'Counter\' after {} calls: {}", signers.clone().len(), num_after);

    Ok(())
}

/// Load contract: `setArray` method
/// NOTE: signer needed as it incurs gas fees.
pub(crate) async fn load_set_array(
    client: Arc<Provider<Http>>,
    load_address: Address,
    signer: Wallet<SigningKey>,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware = SignerMiddleware::new(
        client.clone(),
        signer.with_chain_id(client.get_chainid().await?.as_u64()),
    );

    // clone the client (if multiple use)
    let client_middleware = Arc::new(client_middleware);

    // get a contract
    let load = Load::new(load_address, client_middleware);

    // TODO: Here, `count` can be abstracted out as CLI parameter with default value set as may be `1000`
    // considered the highest possible count per block for now.
    let count = MAX_LOAD_COUNT_PER_BLOCK;

    // send a transaction with setter function
    let tx_receipt = load
        // to try out with its different values.
        // The max. `count` possible in `setArray` method of Load contract is 2650. Above this count value,
        // the gas cost exceeds 60 M per block (as set for Subspace EVM domain).
        .set_array(U256::from(count))
        .send()
        .await
        .expect("Failure in getting pending tx")
        .await?
        .expect("Failure in \'setArray\' method of Load contract");
    println!(
        "\n\'{}\' set Array with count {}, which incurred a gas fee of \'{} TSSC\', has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        tx_receipt.from,
        count,
        get_gas_cost(&client.clone(), &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}

/// Like `handle_async_calls_in_batch_light` but for HEAVY txs.
/// Considered `Load` contract's `setArray` method as HEAVY txs.
async fn handle_async_calls_in_batch_heavy(
    client: Arc<Provider<Http>>,
    load_address: Address,
    signers: Vec<Wallet<SigningKey>>,
) -> eyre::Result<()> {
    // iteration in chunk of `MAX_BATCH_SIZE`
    for chunk in signers.chunks(MAX_BATCH_SIZE.into()) {
        // create a batch vec for this chunk
        let mut batch = Vec::new();

        for signer in chunk {
            batch.push(load_set_array(client.to_owned(), load_address, signer.to_owned()));
        }

        // send txs in batch of 100 (set now, can be adjusted later)
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
) -> eyre::Result<()> {
    // Handle async calls in batches where each batch has `MAX_BATCH_SIZE` requests.
    handle_async_calls_in_batch_heavy(client.to_owned(), load_address, signers.to_owned()).await?;

    Ok(())
}

/// Get contract addresses from env variables from `.env` file
pub(crate) async fn get_contract_addresses_from_env(
) -> eyre::Result<(Address, Address, Address, Address)> {
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

    Ok((counter_address, load_address, multicall_address, fund_address))
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

/// generate new accounts and transfer TSSC
pub(crate) async fn gen_wallets_transfer_tssc(
    client: Arc<Provider<Http>>,
    num_accounts: u32,
    funder_wallet: Wallet<SigningKey>,
    funding_amount: u64,
    fund_contract_addr: Address,
) -> eyre::Result<Vec<Wallet<SigningKey>>> {
    // generate some new accounts and send funds to each of them
    let mut wallet_addresses = Vec::<Address>::new();
    let mut wallet_priv_keys = Vec::<String>::new();
    let mut signers: Vec<Wallet<SigningKey>> = Vec::new();

    // generate multiple accounts based on the parsed number.
    for i in 0..num_accounts {
        let mut rng: rand::rngs::ThreadRng = thread_rng();
        let wallet = LocalWallet::new(&mut rng);
        // println!("Successfully created new keypair.");
        let pub_key = wallet.address();
        println!("\nAddress[{i}]:     {:?}", pub_key);
        // TODO: [OPTIONAL] save the keypair into a local file or show in the output. Create a CLI flag like --to-console/--to-file
        let priv_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));
        println!("Private key[{i}]: {}", priv_key);
        signers.push(wallet);
        wallet_addresses.push(pub_key);
        wallet_priv_keys.push(priv_key);
    }

    println!(
        "\nCalling \'Fund\' contract\'s \'transferTsscToMany\' \nmethod for sending funds in bulk..."
    );

    // M-2: transfer funds using 'Fund' contract
    // Recommended for bulk transfer from single account.
    // Also all transfers added via single tx i.e. funder's single signature.
    transfer_tssc_bulk(
        client.clone(),
        &funder_wallet,
        wallet_addresses.clone(),
        U256::from(funding_amount),
        fund_contract_addr,
    )
    .await?;

    Ok(signers)
}

/// Transfer TSSC in bulk
pub(crate) async fn transfer_tssc_bulk(
    client: Arc<Provider<Http>>,
    from_wallet: &Wallet<SigningKey>,
    tos: Vec<Address>,
    funding_amount: U256,
    fund_contract_addr: Address,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware = SignerMiddleware::new(
        client.clone(),
        from_wallet.clone().with_chain_id(client.get_chainid().await?.as_u64()),
    );

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
        "\n\'{}\' sent funds to newly created accounts, which incurred a gas fee of \'{} TSSC\', has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        tx_receipt.from,
        get_gas_cost(&client.clone(), &tx_receipt).await?,
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
    println!("\n=====\nFunder's final balance: {} TSSC.", funder_balance_tssc_final);
    let spent_bal_tssc = wei_to_tssc_f64(
        funder_balance_wei_initial.checked_sub(funder_balance_wei_final).expect("Invalid sub op."),
    );
    println!("Funder spent: {:.18} TSSC", spent_bal_tssc);

    Ok(())
}
