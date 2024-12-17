use std::borrow::Cow;

pub(in crate::text) const LAST_TX: &str = "app/text/last";

pub(in crate::text) fn text(id: u64) -> String {
    format!("app/text/{id}")
}
