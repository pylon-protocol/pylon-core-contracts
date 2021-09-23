use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, HumanAddr, MigrateResponse, Uint128};
use cosmwasm_storage::Singleton;
use cw20::Cw20ReceiveMsg;
use pylon_gateway::pool_msg::{Cw20HookMsg, HandleMsg, MigrateMsg};
use std::ops::Mul;
use std::str::FromStr;

use crate::contract;
use crate::handler::migrate;
use crate::state::time_range::TimeRange;
use crate::state::{config, reward, user};
use crate::testing::constants::*;
use crate::testing::utils;

const AMOUNT: u64 = 1000000u64;

#[test]
fn migrate_v1() {
    let mut deps = mock_dependencies(20, &[]);
    let owner = utils::initialize(&mut deps);

    Singleton::new(&mut deps.storage, config::KEY_CONFIG)
        .save(&migrate::V1Config {
            owner: deps.api.canonical_address(&owner.message.sender).unwrap(),
            share_token: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_SHARE_TOKEN))
                .unwrap(),
            reward_token: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_REWARD_TOKEN))
                .unwrap(),
            start_time: TEST_POOL_START,
            cliff_time: TEST_POOL_CLIFF,
            finish_time: TEST_POOL_START + TEST_POOL_PERIOD,
            reward_rate: Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap(),
        })
        .unwrap();

    let msg = MigrateMsg::V1 {};
    let res = contract::migrate(&mut deps, owner.clone(), msg)
        .expect("testing: should success to migrate (v1 -> new)");
    assert_eq!(res, MigrateResponse::default());

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            owner: owner.message.sender,
            share_token: HumanAddr::from(TEST_SHARE_TOKEN),
            deposit_config: config::DepositConfig {
                time: TimeRange {
                    start: TEST_POOL_START,
                    finish: 0,
                    inverse: false
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero()
            },
            withdraw_time: vec![TimeRange {
                start: TEST_POOL_START,
                finish: TEST_POOL_START + TEST_POOL_PERIOD,
                inverse: true
            }],
            reward_token: HumanAddr::from(TEST_REWARD_TOKEN),
            claim_time: TimeRange {
                start: TEST_POOL_CLIFF,
                finish: 0,
                inverse: false
            },
            distribution_config: config::DistributionConfig {
                time: TimeRange {
                    start: TEST_POOL_START,
                    finish: TEST_POOL_START + TEST_POOL_PERIOD,
                    inverse: false
                },
                reward_rate: Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap(),
                total_reward_amount: Uint256::from(TEST_POOL_PERIOD)
                    .mul(Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap())
            }
        }
    );

    let mut sender = mock_env(MOCK_CONTRACT_ADDR, &[]);

    /* ================================= DEPOSIT ================================= */
    let deposit_amount = Uint256::from(AMOUNT);
    let msg = HandleMsg::DepositInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: deposit_amount,
    };

    // before start_time
    sender.block.time = TEST_POOL_START - 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle deposit message");
    utils::assert_generic_err("v1_deposit after start_time", err);

    // after start_time
    sender.block.time = TEST_POOL_START + 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle deposit message");

    // before finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD - 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle deposit message");

    // after finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    contract::handle(&mut deps, sender.clone(), msg)
        .expect("testing: should handle deposit message");

    /* ================================= WITHDRAW ================================= */
    let withdraw_amount = Uint256::from(AMOUNT);
    let msg = HandleMsg::WithdrawInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: withdraw_amount,
    };

    // store mock data
    reward::store(
        &mut deps.storage,
        &reward::Reward {
            total_deposit: withdraw_amount.mul(Uint256::from(4u64)),
            last_update_time: 0,
            reward_per_token_stored: Default::default(),
        },
    )
    .unwrap();
    user::store(
        &mut deps.storage,
        &deps
            .api
            .canonical_address(&HumanAddr::from(TEST_USER))
            .unwrap(),
        &user::User {
            amount: withdraw_amount.mul(Uint256::from(4u64)),
            reward: Default::default(),
            reward_per_token_paid: Default::default(),
        },
    )
    .unwrap();

    // before start_time
    sender.block.time = TEST_POOL_START - 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle withdraw message");

    // after start_time
    sender.block.time = TEST_POOL_START + 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle withdraw message");
    utils::assert_generic_err("v1_withdraw after start_time", err);

    // before finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD - 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle withdraw message");
    utils::assert_generic_err("v1_withdraw before finish_time", err);

    // after finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    contract::handle(&mut deps, sender.clone(), msg)
        .expect("testing: should handle withdraw message");

    /* ================================= CLAIM ================================= */
    let msg = HandleMsg::ClaimInternal {
        sender: HumanAddr::from(TEST_USER),
    };

    // before cliff_time
    sender.block.time = TEST_POOL_CLIFF - 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle claim message");
    utils::assert_generic_err("v1_claim after start_time", err);

    // after cliff_time
    sender.block.time = TEST_POOL_CLIFF + 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle claim message");

    // before finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD - 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle claim message");

    // after finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    contract::handle(&mut deps, sender, msg).expect("testing: should handle claim message");
}

#[test]
fn migrate_v1_temp() {
    let mut deps = mock_dependencies(20, &[]);
    let owner = utils::initialize(&mut deps);

    Singleton::new(&mut deps.storage, config::KEY_CONFIG)
        .save(&migrate::V1TempConfig {
            owner: deps.api.canonical_address(&owner.message.sender).unwrap(),
            share_token: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_SHARE_TOKEN))
                .unwrap(),
            reward_token: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_REWARD_TOKEN))
                .unwrap(),
            start_time: TEST_POOL_START,
            cliff_time: TEST_POOL_CLIFF,
            finish_time: TEST_POOL_START + TEST_POOL_PERIOD,
            temp_withdraw_start_time: TEST_POOL_CLIFF,
            temp_withdraw_finish_time: TEST_POOL_CLIFF + (TEST_POOL_PERIOD / 4),
            reward_rate: Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap(),
        })
        .unwrap();

    let msg = MigrateMsg::V1Temp {};
    let res = contract::migrate(&mut deps, owner.clone(), msg)
        .expect("testing: should success to migrate (v1_temp -> new)");
    assert_eq!(res, MigrateResponse::default());

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            owner: owner.message.sender,
            share_token: HumanAddr::from(TEST_SHARE_TOKEN),
            deposit_config: config::DepositConfig {
                time: TimeRange {
                    start: TEST_POOL_START,
                    finish: 0,
                    inverse: false
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero()
            },
            withdraw_time: vec![
                TimeRange {
                    start: TEST_POOL_START,
                    finish: TEST_POOL_START + TEST_POOL_PERIOD,
                    inverse: true
                },
                TimeRange {
                    start: TEST_POOL_CLIFF,
                    finish: TEST_POOL_CLIFF + (TEST_POOL_PERIOD / 4),
                    inverse: false,
                }
            ],
            reward_token: HumanAddr::from(TEST_REWARD_TOKEN),
            claim_time: TimeRange {
                start: TEST_POOL_CLIFF,
                finish: 0,
                inverse: false
            },
            distribution_config: config::DistributionConfig {
                time: TimeRange {
                    start: TEST_POOL_START,
                    finish: TEST_POOL_START + TEST_POOL_PERIOD,
                    inverse: false
                },
                reward_rate: Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap(),
                total_reward_amount: Uint256::from(TEST_POOL_PERIOD)
                    .mul(Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap())
            }
        }
    );

    let mut sender = mock_env(MOCK_CONTRACT_ADDR, &[]);

    /* ================================= DEPOSIT ================================= */
    let deposit_amount = Uint256::from(AMOUNT);
    let msg = HandleMsg::DepositInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: deposit_amount,
    };

    // before start_time
    sender.block.time = TEST_POOL_START - 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle deposit message");
    utils::assert_generic_err("v1_temp_deposit after start_time", err);

    // after start_time
    sender.block.time = TEST_POOL_START + 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle deposit message");

    // before finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD - 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle deposit message");

    // after finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    contract::handle(&mut deps, sender.clone(), msg)
        .expect("testing: should handle deposit message");

    /* ================================= WITHDRAW ================================= */
    let withdraw_amount = Uint256::from(AMOUNT);
    let msg = HandleMsg::WithdrawInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: withdraw_amount,
    };

    // store mock data
    reward::store(
        &mut deps.storage,
        &reward::Reward {
            total_deposit: withdraw_amount.mul(Uint256::from(4u64)),
            last_update_time: 0,
            reward_per_token_stored: Default::default(),
        },
    )
    .unwrap();
    user::store(
        &mut deps.storage,
        &deps
            .api
            .canonical_address(&HumanAddr::from(TEST_USER))
            .unwrap(),
        &user::User {
            amount: withdraw_amount.mul(Uint256::from(4u64)),
            reward: Default::default(),
            reward_per_token_paid: Default::default(),
        },
    )
    .unwrap();

    // before start_time
    sender.block.time = TEST_POOL_START - 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle withdraw message");

    // after start_time
    sender.block.time = TEST_POOL_START + 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle withdraw message");
    utils::assert_generic_err("v1_temp_withdraw after start_time", err);

    // before temp_start_time
    sender.block.time = TEST_POOL_CLIFF - 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle withdraw message");
    utils::assert_generic_err("v1_temp_withdraw before temp_start_time", err);

    // after temp_start_time
    sender.block.time = TEST_POOL_CLIFF + 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle withdraw message");

    // before temp_finish_time
    sender.block.time = TEST_POOL_CLIFF + (TEST_POOL_PERIOD / 4) - 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle withdraw message");

    // after temp_finish_time
    sender.block.time = TEST_POOL_CLIFF + (TEST_POOL_PERIOD / 4) + 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle withdraw message");
    utils::assert_generic_err("v1_temp_withdraw after temp_finish_time", err);

    // before finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD - 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle withdraw message");
    utils::assert_generic_err("v1_temp_withdraw before finish_time", err);

    // after finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    contract::handle(&mut deps, sender.clone(), msg)
        .expect("testing: should handle withdraw message");

    /* ================================= CLAIM ================================= */
    let msg = HandleMsg::ClaimInternal {
        sender: HumanAddr::from(TEST_USER),
    };

    // before cliff_time
    sender.block.time = TEST_POOL_CLIFF - 1;
    let err = contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect_err("testing: should fail to handle claim message");
    utils::assert_generic_err("v1_temp_claim after start_time", err);

    // after cliff_time
    sender.block.time = TEST_POOL_CLIFF + 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle claim message");

    // before finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD - 1;
    contract::handle(&mut deps, sender.clone(), msg.clone())
        .expect("testing: should handle claim message");

    // after finish_time
    sender.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    contract::handle(&mut deps, sender, msg).expect("testing: should handle claim message");
}
