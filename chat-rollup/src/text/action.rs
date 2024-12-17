use astria_eyre::eyre::{ensure, Result, WrapErr as _};
use cnidarium::{StateRead, StateWrite};
use rollup_core::transaction::v1::action::Transfer;

use crate::text::{StateReadExt as _, StateWriteExt as _};

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

pub(crate) async fn execute_send_text<S, String>(
    action: &rollup_core::transaction::v1::action::SendText,
    from: &String,
    mut state: S,
) -> Result<()>
where
    S: StateWrite,
{
    let last_id = state.get_last_text_id().await.unwrap();
    let last_id: u64 = last_id.into();
    state.put_text(action.text.clone(), last_id + 1).unwrap();
    state.put_last_text_id(last_id + 1).unwrap();

    Ok(())
}
