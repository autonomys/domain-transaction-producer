use std::sync::Arc;

use bindings::counter::{self, Counter, COUNTER_ABI};
use bindings::load::{Load, LOAD_ABI};
use ethers::{
    prelude::*,
    signers::Wallet,
    types::transaction::{eip2718::TypedTransaction, eip2930::AccessList},
    utils::format_units,
};
use k256::ecdsa::SigningKey;

/// Transfer TSSC function
pub(crate) async fn transfer_tssc(
    provider: &Provider<Http>,
    from_wallet: &Wallet<SigningKey>,
    to: Address,
    amount: U256,
) -> eyre::Result<()> {
    let from = from_wallet.address();

    // let balance_before = provider.get_balance(from, None).await?;
    let nonce1 = provider.get_transaction_count(from, None).await?;

    // 1. create a tx
    println!("Creating tx...");
    let typed_tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
        from: Some(from),
        to: Some(to.into()),
        gas: Some(U256::from(21000)),
        value: Some(U256::from(amount)),
        data: None,
        nonce: Some(nonce1),
        access_list: AccessList(vec![]),
        max_priority_fee_per_gas: None,
        max_fee_per_gas: Some(provider.get_gas_price().await?),
        chain_id: Some(provider.get_chainid().await?.as_u64().into()),
    });
    // println!("\nTyped tx: {:?}", typed_tx);
    // println!("\nTyped tx hash: {:?}", typed_tx.sighash());

    // 2. sign the tx
    println!("Signing tx...");
    let signature = from_wallet.sign_transaction(&typed_tx).await?;
    // println!("\nSignature: {:?}", signature);

    // 3. serialize the signed tx to get the raw tx
    // RLP encoding has to be done as `Bytes` (ethers::types::Bytes) array
    let rlp_encoded_tx_bytes = typed_tx.rlp_signed(&signature);
    // println!("\nRLP encoded tx bytes: {:?}", rlp_encoded_tx_bytes);

    // 4. send the raw transaction
    println!("Sending raw tx...");
    let tx_receipt = provider
        // `eth_sendRawTransaction` is run
        .send_raw_transaction(rlp_encoded_tx_bytes)
        .await
        .expect("Failure in raw tx [1]")
        .await
        .expect("Failure in raw tx [2]")
        .expect("Failure in getting tx receipt");
    println!(
        "Funds sent to \'{}\', which incurred a gas fee of \'{} TSSC\' has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        to,
        get_gas_cost(provider, &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    let nonce2 = provider.get_transaction_count(from, None).await?;
    assert!(nonce2 > nonce1, "Sender's nonce must be incremented after each tx");

    // CLEANUP: remove later (if not required)
    // let balance_after = provider.get_balance(from, None).await?;
    // assert!(balance_after < balance_before);

    // println!("{} has balance before: {balance_before}", from);
    // println!("{} has balance after: {balance_after}", from);

    Ok(())
}

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

/// sending a bundle of txs
fn bundle_tx() {
    todo!("bundle txs using flashbot mechanism.")
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
    println!("gas cost: {} TSSC", gas_cost_tssc);

    Ok(gas_cost_tssc)
}

/// get Counter number
/// NOTE: No signer needed as it is gasless call.
pub(crate) async fn counter_get_number(
    provider: &'static Provider<Http>,
    counter_address: Address,
) -> eyre::Result<U256> {
    let client = Arc::new(provider);
    let counter = Counter::new(counter_address, client);

    let num = counter.number().call().await?;

    Ok(num)
}
/// set Counter number
/// NOTE: signer needed as it incurs gas fees.
pub(crate) async fn counter_set_number(
    provider: &'static Provider<Http>,
    counter_address: Address,
    signer: Wallet<SigningKey>,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client = SignerMiddleware::new(
        provider,
        signer.with_chain_id(provider.get_chainid().await?.as_u64()),
    );

    // clone the client (if multiple use)
    let client = Arc::new(client);

    // get a contract
    let counter = Counter::new(counter_address, client);

    // send a transaction with setter function
    let tx_receipt = counter
        .set_number(U256::from(42))
        .send()
        .await?
        .await?
        .expect("Failure in \'set_number\' of Counter contract");
    println!(
        "Number set as \'42\', which incurred a gas fee of \'{} TSSC\' has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        get_gas_cost(provider, &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}

/// increment Counter number
/// NOTE: signer needed as it incurs gas fees.
pub(crate) async fn counter_increment(
    provider: &'static Provider<Http>,
    counter_address: Address,
    signer: Wallet<SigningKey>,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client = SignerMiddleware::new(
        provider,
        signer.with_chain_id(provider.get_chainid().await?.as_u64()),
    );

    // clone the client (if multiple use)
    let client = Arc::new(client);

    // get a contract
    let counter = Counter::new(counter_address, client);

    // send a transaction with setter function
    let tx_receipt = counter
        .increment()
        .send()
        .await?
        .await?
        .expect("Failure in \'increment\' of Counter contract");
    println!(
        "Number set as \'42\', which incurred a gas fee of \'{} TSSC\' has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        get_gas_cost(provider, &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}

/// As LIGHT transaction type, multicall particular function
/// let's say `increment` successively done by each new accounts w/o
/// `num_block` cli arg.
pub(crate) async fn multicall_light_txs(
    provider: &'static Provider<Http>,
    counter_address: Address,
    signers: Vec<Wallet<SigningKey>>,
) {
    // TODO:
}

/// As HEAVY transaction type, multicall particular function
/// let's say `increment` successively done by each new accounts w/o
/// `num_block` cli arg.
pub(crate) async fn multicall_heavy_txs(
    provider: &'static Provider<Http>,
    load_address: Address,
    signers: Vec<Wallet<SigningKey>>,
) {
    // TODO:
}
