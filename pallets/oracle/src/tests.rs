use crate::mock::*;
use frame_support::{assert_err, assert_ok};

const ALICE: u64 = 1u64;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		assert_ok!(PalletOracle::authorize(Origin::root(), ALICE));
		assert_err!(
			PalletOracle::handle_event(Origin::root(), vec![]),
			sp_runtime::traits::BadOrigin
		);

		assert_ok!(PalletOracle::handle_event(Origin::signed(ALICE), vec![]));

		for _i in 0..2000 {
			assert_ok!(PalletOracle::handle_event(Origin::signed(ALICE), vec![]));
		}
		let storage = PalletOracle::event_storage();
		let len = storage.events.len();
		assert_eq!(len, 1000);
	});
}
