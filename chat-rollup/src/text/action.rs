#![allow(unused_imports)]
use astria_eyre::eyre::{ensure, Result, WrapErr as _};
use cnidarium::{StateRead, StateWrite};
use rollup_core::transaction::v1::action::Transfer;

use super::AddressBytes;
use crate::text::{StateReadExt as _, StateWriteExt as _};

#[allow(unused_imports)]
use crate::{
    accounts::{StateReadExt as _, StateWriteExt as _},
    address::{StateReadExt as _, StateReadExt as _},
};
use tracing::warn;

// #[async_trait::async_trait]
// impl ActionHandler for Transfer {
//     async fn check_stateless(&self) -> Result<()> {
//         Ok(())
//     }

//     async fn check_and_execute<S: StateWrite>(&self, state: S) -> Result<()> {
//         let from = state
//             .get_transaction_context()
//             .expect("transaction source must be present in state when executing an action")
//             .address_bytes();

//         ensure!(
//             state
//                 .get_bridge_account_rollup_id(&from)
//                 .await
//                 .wrap_err("failed to get bridge account rollup id")?
//                 .is_none(),
//             "cannot transfer out of bridge account; BridgeUnlock must be used",
//         );

//         check_transfer(self, &from, &state).await?;
//         execute_transfer(self, &from, state).await?;

//         Ok(())
//     }
// }

pub(crate) async fn execute_send_text<S, TAddress>(
    action: &rollup_core::transaction::v1::action::SendText,
    from: &TAddress,
    mut state: S,
) -> Result<()>
where
    S: StateWrite,
    TAddress: AddressBytes,
{
    warn!("execute_send_text");
    warn!("execute_send_text");
    warn!("execute_send_text");
    warn!("execute_send_text");
    warn!("execute_send_text");
    let from = from.address_bytes();

    let last_id = state.get_last_text_id().await.unwrap();
    let last_id: u64 = last_id.into();
    state
        .put_text(action.text.clone(), action.from.clone(), last_id)
        .unwrap();
    state.put_last_text_id(last_id + 1).unwrap();

    // TODO: Implement configurable fees and fee recipients
    state
        .decrease_balance(from, &action.fee_asset, 1)
        .await
        .wrap_err("failed decreasing `from` account balance")?;

    let nonce = state
        .get_account_nonce(from)
        .await
        .wrap_err("failed to get account nonce")?;
    let nonce = nonce + 1;
    state
        .put_account_nonce(from, nonce)
        .wrap_err("failed to put account nonce")?;
    Ok(())
}
