use crate::utils::wei_to_tssc_f64;
use bindings::{counter::Counter, load::Load};
use ethers::{core::k256::ecdsa::SigningKey, prelude::*, signers::Wallet};
use log::debug;
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

    // get a contract
    let counter = Counter::new(counter_address, Arc::new(client_middleware));

    // send a transaction with setter function
    let tx_receipt = counter
        .set_number(U256::from(42))
        .send()
        .await?
        .await?
        .expect("Failure in \'setNumber\' method of Counter contract");
    log_tx_dbg(tx_receipt, format!("Counter::setNumber({})", 42).as_str());

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

    // get a contract
    let counter = Counter::new(counter_address, Arc::new(client_middleware));

    // send a transaction with setter function
    let tx_receipt = counter
        .increment()
        .send()
        .await
        .expect("Failure in getting pending tx")
        .await?
        .expect("Failure in \'increment\' method of Counter contract");

    log_tx_dbg(tx_receipt, "Counter::increment()");

    Ok(())
}

/// Get balance of address
async fn get_balance(client: Arc<Provider<Http>>, of: Address) -> eyre::Result<U256> {
    let balance = client
        .get_balance(of, None)
        .await
        .expect(format!("Failed to get the balance of {}", of).as_str());

    Ok(balance)
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
    let client_middleware =
        SignerMiddleware::new(client.clone(), signer.clone().with_chain_id(chain_id));

    // get a contract
    let load = Load::new(load_address, Arc::new(client_middleware));

    // TODO: Here, `count` can be abstracted out as CLI parameter with default value set as may be `1000`
    // considered the highest possible count per block for now.
    let count = max_load_count_per_block;

    // check for estimated balance
    let from_balance_pre = get_balance(client, signer.address()).await?;
    let estimated_gas = load.set_array(U256::from(count)).estimate_gas().await?;
    debug!("Estimated gas: {}", estimated_gas);
    // set gas price to 3.5 Gwei for heavy tx type
    let estimated_gas_price = estimated_gas.checked_mul(U256::from(3500000000_u32)).unwrap();
    assert!(
        from_balance_pre >= estimated_gas_price,
        "Balance short by: {}",
        estimated_gas_price
            .checked_sub(from_balance_pre)
            .expect("[Load] Error in subtracting bal. from est. gas price"),
    );

    debug!("[Pre-tx] Est. gas price: {}", estimated_gas_price,);

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

    // need to check the gas unit if that also changes each call.
    debug!("[Post-tx] Gas consumed: {}", tx_receipt.gas_used.unwrap_or_default());
    // log all details regarding the tx
    log_tx_dbg(tx_receipt, format!("Load::setArray({})", count).as_str());

    Ok(())
}

/// debug! tx details with custom str
fn log_tx_dbg(tx_receipt: TransactionReceipt, contract_name: &str) {
    let message =
        format!(
        "{} ==> from: {}, gas price: {:.18} TSSC, tx hash: {:?}, tx index: {}, block number: {}",
        contract_name,
        tx_receipt.from,
        wei_to_tssc_f64(tx_receipt
            .effective_gas_price
            .unwrap()
            .checked_mul(tx_receipt.gas_used.unwrap_or_default())
            .unwrap_or(U256::zero())),
        tx_receipt.transaction_hash,
        tx_receipt.transaction_index,
        tx_receipt.block_number.unwrap()
    );

    debug!("{}", message);
}
