// use astria_core::Protobuf;
// use clap_stdin::FileOrStdin;
// use color_eyre::eyre::{self, ensure, WrapErr as _};
// use uprollup::{self, protocol::transaction::v1::Transaction};

// use crate::utils::submit_transaction;
// #[derive(clap::Args, Debug)]
// pub(super) struct Command {
//     /// The URL at which the Sequencer node is listening for ABCI commands.
//     #[arg(
//         long,
//         env = "SEQUENCER_URL",
//         default_value = crate::DEFAULT_SEQUENCER_RPC
//     )]
//     sequencer_url: String,
//     /// The source to read the pbjson formatted astra.protocol.transaction.v1.Transaction (use `-`
//     /// to pass via STDIN).
//     input: FileOrStdin,
// }

// // The 'submit' command takes a 'Transaction' in pbjson form and submits it to the sequencer
// impl Command {
//     pub(super) async fn run(self) -> eyre::Result<()> {
//         let filename = self.input.filename().to_string();
//         let transaction = read_transaction(self.input)
//             .wrap_err_with(|| format!("to signed transaction from `{filename}`"))?;
//         submit_transaction(
//             self.sequencer_url.as_str(),
//             self.,
//             prefix,
//             private_key,
//             action,
//         )
//         .await
//         .unwrap();
//         let json_tx = serde_json::to_string(&transaction.to_raw())
//             .wrap_err("failed to write signed transaction")?;
//         println!("Submitting transaction: {}", json_tx);
//         println!("Submission completed!");
//         Ok(())
//     }
// }

// fn read_transaction(input: FileOrStdin) -> eyre::Result<Transaction> {
//     let wire_body: <Transaction as Protobuf>::Raw = serde_json::from_reader(
//         std::io::BufReader::new(input.into_reader()?),
//     )
//     .wrap_err_with(|| {
//         format!(
//             "failed to parse input as json `{}`",
//             Transaction::full_name()
//         )
//     })?;
//     Transaction::try_from_raw(wire_body).wrap_err("failed to validate transaction body")
// }
