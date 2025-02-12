use astria_core::crypto::SigningKey;
use astria_core::primitive::v1::Address;
use astria_core::Protobuf;
use rollup_core::transaction::v1::{Action, TransactionBody};

use color_eyre::eyre::{self, eyre, WrapErr as _};
use prost::Message;
use reqwest::Response;
use rollup_core::generated::protocol::transaction::v1::Transaction;

pub(crate) async fn submit_transaction(
    rollup_url: &str,
    chain_id: String,
    prefix: &str,
    private_key: &str,
    action: Action,
) -> eyre::Result<Response> {
    let sequencer_key = signing_key_from_private_key(private_key)?;

    let from_address = address_from_signing_key(&sequencer_key, prefix)?;
    println!("sending tx from address: {from_address}");

    let response = reqwest::Client::new()
        .get(format!("{}/get_account_nonce/{}", rollup_url, from_address))
        .send()
        .await?;
    let text = response.text().await?;
    // Remove the surrounding quotes and then parse
    let nonce = text.trim_matches('"').parse::<u32>().map_err(|e| {
        println!("Failed to parse '{}' as nonce: {}", text, e);
        e
    })?;

    let tx = TransactionBody::builder()
        .nonce(nonce)
        .chain_id(chain_id)
        .actions(vec![action])
        .try_build()
        .wrap_err("failed to construct a transaction")?
        .sign(&sequencer_key);
    let encoded: prost::bytes::Bytes = tx.into_raw().encode_to_vec().into();
    let decoded = Transaction::decode(encoded.clone()).wrap_err("failed to decode transaction")?;
    println!("decoded transaction: {:?}", decoded);
    println!("encoded transaction: {:?}", encoded);
    // Send the POST request
    let response = reqwest::Client::new()
        .post(format!("{}/submit_transaction", rollup_url))
        .body(encoded)
        .send()
        .await
        .unwrap();

    Ok(response)
}

pub(crate) fn signing_key_from_private_key(private_key: &str) -> eyre::Result<SigningKey> {
    // Decode the hex string to get the private key bytes
    let private_key_bytes: [u8; 32] = hex::decode(private_key)
        .wrap_err("failed to decode private key bytes from hex string")?
        .try_into()
        .map_err(|_| eyre!("invalid private key length; must be 32 bytes"))?;

    // Create and return a signing key from the private key bytes
    Ok(SigningKey::from(private_key_bytes))
}

pub(crate) fn address_from_signing_key(
    signing_key: &SigningKey,
    prefix: &str,
) -> eyre::Result<Address> {
    // Build the address using the public key from the signing key
    let from_address = Address::builder()
        .array(*signing_key.verification_key().address_bytes())
        .prefix(prefix)
        .try_build()
        .wrap_err("failed constructing a valid from address from the provided prefix")?;

    // Return the generated address
    Ok(from_address)
}
