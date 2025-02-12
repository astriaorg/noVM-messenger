// //! Utilities to create objects used in various tests of the Astria codebase.

// use std::collections::HashMap;

// use bytes::Bytes;
// use prost::Message as _;

// use super::{
//     group_rollup_data_submissions_in_signed_transaction_transactions_by_rollup_id,
//     transaction::v1::action::RollupDataSubmission,
// };
// use crate::{
//     crypto::SigningKey,
//     primitive::v1::{derive_merkle_tree_from_rollup_txs, RollupId},
//     protocol::transaction::v1::TransactionBody,
//     sequencerblock::v1::{block::Deposit, SequencerBlock},
//     Protobuf as _,
// };

// #[derive(Default)]
// pub struct UnixTimeStamp {
//     pub secs: i64,
//     pub nanos: u32,
// }

// impl From<(i64, u32)> for UnixTimeStamp {
//     fn from(val: (i64, u32)) -> Self {
//         Self {
//             secs: val.0,
//             nanos: val.1,
//         }
//     }
// }

// /// Allows configuring a Comet BFT block setting the height, signing key and
// /// proposer address.
// ///
// /// If the proposer address is not set it will be generated from the signing key.
// #[derive(Default)]
// pub struct ConfigureSequencerBlock {
//     pub block_hash: Option<[u8; 32]>,
//     pub chain_id: Option<String>,
//     pub height: u32,
//     pub proposer_address: Option<tendermint::account::Id>,
//     pub signing_key: Option<SigningKey>,
//     pub sequence_data: Vec<(RollupId, Vec<u8>)>,
//     pub deposits: Vec<Deposit>,
//     pub unix_timestamp: UnixTimeStamp,
// }

// impl ConfigureSequencerBlock {
//     /// Construct a [`SequencerBlock`] with the configured parameters.
//     #[must_use]
//     #[expect(
//         clippy::missing_panics_doc,
//         reason = "This should only be used in tests, so everything here is unwrapped"
//     )]
//     pub fn make(self) -> SequencerBlock {
//         use tendermint::Time;

//         use crate::{protocol::transaction::v1::Action, sequencerblock::v1::block::RollupData};

//         let Self {
//             block_hash,
//             chain_id,
//             height,
//             signing_key,
//             proposer_address,
//             sequence_data,
//             unix_timestamp,
//             deposits,
//         } = self;

//         let block_hash = block_hash.unwrap_or_default();
//         let chain_id = chain_id.unwrap_or_else(|| "test".to_string());

//         let signing_key = signing_key.unwrap_or_else(|| SigningKey::new(rand::rngs::OsRng));

//         let proposer_address = proposer_address.unwrap_or_else(|| {
//             let public_key: tendermint::crypto::ed25519::VerificationKey =
//                 signing_key.verification_key().as_ref().try_into().unwrap();
//             tendermint::account::Id::from(public_key)
//         });

//         let actions: Vec<Action> = sequence_data
//             .into_iter()
//             .map(|(rollup_id, data)| {
//                 RollupDataSubmission {
//                     rollup_id,
//                     data: data.into(),
//                     fee_asset: "nria".parse().unwrap(),
//                 }
//                 .into()
//             })
//             .collect();
//         let txs = if actions.is_empty() {
//             vec![]
//         } else {
//             let body = TransactionBody::builder()
//                 .actions(actions)
//                 .chain_id(chain_id.clone())
//                 .nonce(1)
//                 .try_build()
//                 .expect(
//                     "should be able to build transaction body since only rollup data submission \
//                      actions are contained",
//                 );
//             vec![body.sign(&signing_key)]
//         };
//         let mut deposits_map: HashMap<RollupId, Vec<Deposit>> = HashMap::new();
//         for deposit in deposits {
//             if let Some(entry) = deposits_map.get_mut(&deposit.rollup_id) {
//                 entry.push(deposit);
//             } else {
//                 deposits_map.insert(deposit.rollup_id, vec![deposit]);
//             }
//         }

//         let mut rollup_transactions =
//             group_rollup_data_submissions_in_signed_transaction_transactions_by_rollup_id(&txs);
//         for (rollup_id, deposit) in deposits_map.clone() {
//             rollup_transactions
//                 .entry(rollup_id)
//                 .or_default()
//                 .extend(deposit.into_iter().map(|deposit| {
//                     RollupData::Deposit(Box::new(deposit))
//                         .into_raw()
//                         .encode_to_vec()
//                         .into()
//                 }));
//         }
//         rollup_transactions.sort_unstable_keys();
//         let rollup_transactions_tree = derive_merkle_tree_from_rollup_txs(&rollup_transactions);

//         let rollup_ids_root = merkle::Tree::from_leaves(
//             rollup_transactions
//                 .keys()
//                 .map(|rollup_id| rollup_id.as_ref().to_vec()),
//         )
//         .root();
//         let mut data = vec![
//             rollup_transactions_tree.root().to_vec(),
//             rollup_ids_root.to_vec(),
//         ];
//         data.extend(txs.into_iter().map(|tx| tx.into_raw().encode_to_vec()));
//         let data = data.into_iter().map(Bytes::from).collect();
//         SequencerBlock::try_from_block_info_and_data(
//             block_hash,
//             chain_id.try_into().unwrap(),
//             height.into(),
//             Time::from_unix_timestamp(unix_timestamp.secs, unix_timestamp.nanos).unwrap(),
//             proposer_address,
//             data,
//             deposits_map,
//         )
//         .unwrap()
//     }
// }
