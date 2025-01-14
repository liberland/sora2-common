// This file is part of the SORA network and Polkaswap app.

// Copyright (c) 2020, 2021, Polka Biome Ltd. All rights reserved.
// SPDX-License-Identifier: BSD-4-Clause

// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:

// Redistributions of source code must retain the above copyright notice, this list
// of conditions and the following disclaimer.
// Redistributions in binary form must reproduce the above copyright notice, this
// list of conditions and the following disclaimer in the documentation and/or other
// materials provided with the distribution.
//
// All advertising materials mentioning features or use of this software must display
// the following acknowledgement: This product includes software developed by Polka Biome
// Ltd., SORA, and Polkaswap.
//
// Neither the name of the Polka Biome Ltd. nor the names of its contributors may be used
// to endorse or promote products derived from this software without specific prior written permission.

// THIS SOFTWARE IS PROVIDED BY Polka Biome Ltd. AS IS AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL Polka Biome Ltd. BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
// BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS;
// OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use super::*;
use codec::MaxEncodedLen;
use currencies::BasicCurrencyAdapter;

use bridge_types::traits::OutboundChannel;
use frame_support::traits::{Everything, GenesisBuild};
use frame_support::{assert_noop, assert_ok, parameter_types, Deserialize, Serialize};
use frame_system::RawOrigin;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Keccak256, Verify};
use sp_runtime::{AccountId32, DispatchError, MultiSignature};
use sp_std::convert::From;
use traits::parameter_type_with_key;

use crate::outbound as bridge_outbound_channel;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

const BASE_NETWORK_ID: SubNetworkId = SubNetworkId::Mainnet;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage},
        Tokens: tokens::{Pallet, Call, Config<T>, Storage, Event<T>},
        Currencies: currencies::{Pallet, Call, Storage},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        BridgeOutboundChannel: bridge_outbound_channel::{Pallet, Call, Config<T>, Storage, Event<T>},
    }
);

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(
    Encode,
    Decode,
    PartialEq,
    Eq,
    RuntimeDebug,
    Clone,
    Copy,
    MaxEncodedLen,
    TypeInfo,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub enum AssetId {
    XOR,
    ETH,
    DAI,
}

pub type Balance = u128;
pub type Amount = i128;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<65536>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 0;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
        0
    };
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
}

impl tokens::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = AssetId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type CurrencyHooks = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ();
    type DustRemovalWhitelist = Everything;
}

impl currencies::Config for Test {
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Test, Balances, Amount, u64>;
    type GetNativeCurrencyId = GetBaseAssetId;
    type WeightInfo = ();
}
parameter_types! {
    pub const GetBaseAssetId: AssetId = AssetId::XOR;
    pub GetTeamReservesAccountId: AccountId = AccountId32::from([0; 32]);
    pub GetFeeAccountId: AccountId = AccountId32::from([1; 32]);
    pub GetTreasuryAccountId: AccountId = AccountId32::from([2; 32]);
}

parameter_types! {
    pub const MaxMessagePayloadSize: u64 = 128;
    pub const MaxMessagesPerCommit: u64 = 5;
}

impl bridge_outbound_channel::Config for Test {
    const INDEXING_PREFIX: &'static [u8] = b"commitment";
    type RuntimeEvent = RuntimeEvent;
    type Hashing = Keccak256;
    type MaxMessagePayloadSize = MaxMessagePayloadSize;
    type MaxMessagesPerCommit = MaxMessagesPerCommit;
    type FeeCurrency = GetBaseAssetId;
    type FeeAccountId = GetFeeAccountId;
    type MessageStatusNotifier = ();
    type AuxiliaryDigestHandler = ();
    type WeightInfo = ();
    type Currency = Currencies;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = ();
}

pub fn new_tester() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let config: bridge_outbound_channel::GenesisConfig<Test> =
        bridge_outbound_channel::GenesisConfig {
            interval: 10u32.into(),
            fee: 100u32.into(),
        };
    config.assimilate_storage(&mut storage).unwrap();

    let bob: AccountId = Keyring::Bob.into();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(bob, 1u32.into())],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();

    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn test_submit() {
    new_tester().execute_with(|| {
        let who: AccountId = Keyring::Bob.into();

        // Deposit enough money to cover fees
        Currencies::deposit(AssetId::XOR, &who, 300u32.into()).unwrap();

        assert_ok!(BridgeOutboundChannel::submit(
            BASE_NETWORK_ID,
            &RawOrigin::Signed(who.clone()),
            &[0, 1, 2],
            ()
        ));
        assert_eq!(<ChannelNonces<Test>>::get(BASE_NETWORK_ID), 1);

        assert_ok!(BridgeOutboundChannel::submit(
            BASE_NETWORK_ID,
            &RawOrigin::Signed(who),
            &[0, 1, 2],
            ()
        ));
        assert_eq!(<ChannelNonces<Test>>::get(BASE_NETWORK_ID), 2);
    });
}

#[test]
#[ignore]
fn test_submit_fees_burned() {
    new_tester().execute_with(|| {
        let who: AccountId = Keyring::Bob.into();

        // Deposit enough money to cover fees
        Currencies::deposit(AssetId::XOR, &who, 300u32.into()).unwrap();
        let old_balance = Currencies::free_balance(AssetId::XOR, &who);

        assert_ok!(BridgeOutboundChannel::submit(
            BASE_NETWORK_ID,
            &RawOrigin::Signed(who.clone()),
            &[0, 1, 2],
            ()
        ));
        assert_eq!(
            Currencies::free_balance(AssetId::XOR, &who),
            old_balance - 100
        );
    })
}

#[test]
#[ignore]
fn test_submit_not_enough_funds() {
    new_tester().execute_with(|| {
        let who: AccountId = Keyring::Bob.into();

        Currencies::deposit(AssetId::XOR, &who, 50u32.into()).unwrap();

        assert_noop!(
            BridgeOutboundChannel::submit(BASE_NETWORK_ID, &RawOrigin::Signed(who), &[0, 1, 2], ()),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
    })
}

#[test]
fn test_submit_exceeds_queue_limit() {
    new_tester().execute_with(|| {
        let who: AccountId = Keyring::Bob.into();

        // Deposit enough money to cover fees
        Currencies::deposit(AssetId::XOR, &who, 1000u32.into()).unwrap();

        let max_messages = MaxMessagesPerCommit::get();
        (0..max_messages).for_each(|_| {
            BridgeOutboundChannel::submit(
                BASE_NETWORK_ID,
                &RawOrigin::Signed(who.clone()),
                &[0, 1, 2],
                (),
            )
            .unwrap();
        });

        assert_noop!(
            BridgeOutboundChannel::submit(BASE_NETWORK_ID, &RawOrigin::Signed(who), &[0, 1, 2], ()),
            Error::<Test>::QueueSizeLimitReached,
        );
    })
}

#[test]
fn test_set_fee_not_authorized() {
    new_tester().execute_with(|| {
        let bob: AccountId = Keyring::Bob.into();
        assert_noop!(
            BridgeOutboundChannel::set_fee(RuntimeOrigin::signed(bob), 1000u32.into()),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_submit_exceeds_payload_limit() {
    new_tester().execute_with(|| {
        let who: AccountId = Keyring::Bob.into();

        let max_payload_bytes = MaxMessagePayloadSize::get();
        let payload: Vec<u8> = (0..).take(max_payload_bytes as usize + 1).collect();

        assert_noop!(
            BridgeOutboundChannel::submit(
                BASE_NETWORK_ID,
                &RawOrigin::Signed(who),
                payload.as_slice(),
                ()
            ),
            Error::<Test>::PayloadTooLarge,
        );
    })
}

#[test]
fn test_submit_fails_on_nonce_overflow() {
    new_tester().execute_with(|| {
        let who: AccountId = Keyring::Bob.into();

        <ChannelNonces<Test>>::insert(BASE_NETWORK_ID, u64::MAX);
        assert_noop!(
            BridgeOutboundChannel::submit(BASE_NETWORK_ID, &RawOrigin::Signed(who), &[0, 1, 2], ()),
            Error::<Test>::Overflow,
        );
    });
}
