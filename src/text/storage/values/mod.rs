mod last_text;
mod text;
use borsh::{BorshDeserialize, BorshSerialize};

pub(in crate::text) use self::last_text::LastText;
pub(in crate::text) use self::text::Text;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub(crate) struct Value(ValueImpl);

#[derive(Debug, BorshSerialize, BorshDeserialize)]
enum ValueImpl {
    Text(Text),
    LastText(LastText),
}
