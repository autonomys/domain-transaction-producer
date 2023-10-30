use ethers::{
    prelude::*,
    signers::Wallet,
    types::transaction::{eip2718::TypedTransaction, eip2930::AccessList},
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
    let signature = from_wallet.sign_transaction(&typed_tx).await?;
    println!("\nSignature: {:?}", signature);

    // 3. serialize the signed tx to get the raw tx
    // RLP encoding has to be done as `Bytes` (ethers::types::Bytes) array
    let rlp_encoded_tx_bytes = typed_tx.rlp_signed(&signature);
    // println!("\nRLP encoded tx bytes: {:?}", rlp_encoded_tx_bytes);

    // 4. send the raw transaction
    let tx_receipt = provider
        // `eth_sendRawTransaction` is run
        .send_raw_transaction(rlp_encoded_tx_bytes)
        .await
        .expect("Failure in raw tx [1]")
        .await
        .expect("Failure in raw tx [2]")
        .expect("Failure in getting tx receipt");
    println!(
        "Transaction sent with hash: {}",
        tx_receipt.transaction_hash
    );
    let nonce2 = provider.get_transaction_count(from, None).await?;

    assert!(nonce1 < nonce2);

    // let balance_after = provider.get_balance(from, None).await?;
    // assert!(balance_after < balance_before);

    // println!("{} has balance before: {balance_before}", from);
    // println!("{} has balance after: {balance_after}", from);

    Ok(())
}
