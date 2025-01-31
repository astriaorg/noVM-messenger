use crate::transaction::v1::action::{
    group::{Actions, ErrorKind, Group},
    Action, Transfer,
};
use astria_core::primitive::v1::{asset::Denom, Address};
const ASTRIA_ADDRESS_PREFIX: &str = "astria";

#[test]
fn try_from_list_of_actions_bundleable_general() {
    let address: Address<_> = Address::builder()
        .array([0; 20])
        .prefix(ASTRIA_ADDRESS_PREFIX)
        .try_build()
        .unwrap();

    let asset: Denom = "nria".parse().unwrap();
    let actions = vec![Action::Transfer(Transfer {
        to: address,
        amount: 100,
        asset: asset.clone(),
        fee_asset: asset.clone(),
    })];

    assert!(matches!(
        Actions::try_from_list_of_actions(actions).unwrap().group(),
        Group::BundleableGeneral
    ));
}
#[test]
fn from_list_of_actions_empty() {
    let error_kind = Actions::try_from_list_of_actions(vec![]).unwrap_err().0;
    assert!(
        matches!(error_kind, ErrorKind::Empty { .. }),
        "expected ErrorKind::Empty, got {error_kind:?}"
    );
}

#[test]
fn should_be_in_expected_order() {
    assert!(Group::UnbundleableSudo < Group::BundleableSudo);
    assert!(Group::BundleableSudo < Group::UnbundleableGeneral);
    assert!(Group::UnbundleableGeneral < Group::BundleableGeneral);
}
