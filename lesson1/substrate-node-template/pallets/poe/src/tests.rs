use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		// Dispatch a signed extrinsic.
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		// Read pallet storage and assert an expected result.
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1,frame_system::Pallet::<Test>::block_number()))
		);

	});
}


#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		// Dispatch a signed extrinsic.
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		// Read pallet storage and assert an expected result.
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyClaimed
		);

	});
}


//验证撤销证书操作成功
#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		// Dispatch a signed extrinsic.
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));

	});
}

//撤销不存在的证书操作失败
#[test]
fn revoke_claim_failed_when_claim_notexist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1,2,3,4];
		
		let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::NoSuchProof
		);

	});
}


//转移存证操作成功
#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1,2,3];
		// Dispatch a signed extrinsic.
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), 2, claim.clone()));

	});
}

//转移不属于自己的存在操作失败
#[test]
fn transfer_claim_failed_when_claim_not_Owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1,2,3,4];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(3), 2, claim.clone()),
			Error::<Test>::NotProofOwner
		);

	});
}

//转移不存在的存证操作失败
#[test]
fn transfer_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let claim2 = vec![0,1,2,3,4];
		let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim2.clone()).unwrap();

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), 2, claim2.clone()),
			Error::<Test>::NoSuchProof
		);

	});
}


