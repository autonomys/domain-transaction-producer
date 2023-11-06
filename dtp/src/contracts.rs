use crate::utils::get_gas_cost;
use bindings::{counter::Counter, load::Load};
use ethers::{core::k256::ecdsa::SigningKey, prelude::*, signers::Wallet};
use log::info;
use std::sync::Arc;

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
    chain_id: u64,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware = SignerMiddleware::new(client.clone(), signer.with_chain_id(chain_id));

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
    info!(
        "\n\'{}\' set number as \'42\', which incurred a gas fee of \'{} TSSC\' has a tx hash: \'{:?}\', indexed at #{} in block #{}.",
        tx_receipt.from,
        get_gas_cost(&client, &tx_receipt).await?,
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
    chain_id: u64,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware = SignerMiddleware::new(client.clone(), signer.with_chain_id(chain_id));

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
    info!(
        "\'{}\' increment number, which incurred a gas fee of \'{} TSSC\', has a tx hash: \'{:?}\', indexed at #{} in block #{}.\n",
        tx_receipt.from,
        get_gas_cost(&client, &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}

/// Load contract: `setArray` method
/// NOTE: signer needed as it incurs gas fees.
pub(crate) async fn load_set_array(
    client: Arc<Provider<Http>>,
    load_address: Address,
    signer: Wallet<SigningKey>,
    chain_id: u64,
    max_load_count_per_block: u16,
) -> eyre::Result<()> {
    // create a middleware client with signature from signer & provider
    let client_middleware = SignerMiddleware::new(client.clone(), signer.with_chain_id(chain_id));

    // clone the client (if multiple use)
    let client_middleware = Arc::new(client_middleware);

    // get a contract
    let load = Load::new(load_address, client_middleware);

    // TODO: Here, `count` can be abstracted out as CLI parameter with default value set as may be `1000`
    // considered the highest possible count per block for now.
    let count = max_load_count_per_block;

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

    info!(
        "\'{}\' set Array with count {}, which incurred a gas fee of \'{} TSSC\', has a tx hash: \'{:?}\', indexed at #{} in block #{}.\n",
        tx_receipt.from,
        count,
        get_gas_cost(&client, &tx_receipt).await?,
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    Ok(())
}
