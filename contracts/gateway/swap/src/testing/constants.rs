pub const TEST_BASE_DENOM: u128 = 1000000u128;

//======================= ACCOUNTS
pub const TEST_OWNER: &str = "owner";
pub const TEST_USER_1: &str = "user_1";
pub const TEST_USER_2: &str = "user_2";
pub const TEST_BENEFICIARY: &str = "beneficiary";

//======================= SALE CONFIGURATION
pub const TEST_POOL_X_DENOM: &str = "uusd";
pub const TEST_POOL_Y_ADDR: &str = "token_y";
pub const TEST_POOL_LIQ_X: u128 = 250000 * TEST_BASE_DENOM;
pub const TEST_POOL_LIQ_Y: u128 = 25000000 * TEST_BASE_DENOM;
pub const TEST_STRATEGY_LOCKUP_PERCENT: u64 = 25;
pub const TEST_STRATEGY_VESTING_PERCENT: u64 = 75;
pub const TEST_PRICE: &str = "0.01";
pub const TEST_SWAP_POOL_SIZE: u128 = 100000000u128 * TEST_BASE_DENOM;
