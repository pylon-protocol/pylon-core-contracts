pub const TEST_BASE_DENOM: u128 = 1000000u128;

//======================= ACCOUNTS
pub const TEST_OWNER: &str = "owner";
pub const TEST_USER: &str = "user";
pub const TEST_BENEFICIARY: &str = "beneficiary";

//======================= SALE CONFIGURATION
pub const TEST_POOL_X_DENOM: &str = "uusd";
pub const TEST_POOL_Y_ADDR: &str = "token_y";
pub const TEST_POOL_LIQ_X: u128 = 250000 * TEST_BASE_DENOM;
pub const TEST_POOL_LIQ_Y: u128 = 25000000 * TEST_BASE_DENOM;
pub const TEST_BASE_PRICE: &str = "0.01";
pub const TEST_MIN_USER_CAP: u128 = 200000u128 * TEST_BASE_DENOM;
pub const TEST_MAX_USER_CAP: u128 = 1000000u128 * TEST_BASE_DENOM;
pub const TEST_STAKING: &str = "staking";
pub const TEST_MIN_STAKE_AMOUNT: u128 = 10000u128 * TEST_BASE_DENOM;
pub const TEST_MAX_STAKE_AMOUNT: u128 = 100000u128 * TEST_BASE_DENOM;
pub const TEST_ADDITIONAL_CAP_PER_TOKEN: &str = "8.888888889";
pub const TEST_TOTAL_SALE_AMOUNT: u128 = 100000000u128 * TEST_BASE_DENOM;
