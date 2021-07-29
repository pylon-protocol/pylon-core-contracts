# Lockup

## ExecuteMsg

### Update

- updates accumulated reward per token

**Request**

```jsx
{
	update: {}
}
```

### Redeem // CosmWasm CW-20 `send` message

- locks DP tokens to this contract.
- must be included with the DP token's CW-20 `send` message.
- encode relevant `json` messages in `base64` format

**Request**

```jsx
{
	deposit: {}
}

// example
{
	send: {
		//...
		msg: { // base64 format
			deposit: {}
		}
	}
}
```

**Log**

```jsx
[
	{key: "action", value: "deposit"},
	{key: "sender", value: "{address}"},
	{key: "deposit_amount", value: "{amount}"}
]
```

### Withdraw

- withdraws DP tokens from this contract.

**Request**

- `amount`:  amount to withdraw

```jsx
{
	withdraw: {
		amount: 10000000 // Uint256 - 6 decimals
	}
}
```

**Log**

```jsx
[
	{key: "action", value: "withdraw"},
	{key: "sender", value: "{address}"},
	{key: "withdraw_amount", value: "{amount}"}
]
```

### Claim

- claim all accumulated rewards, denominated in UST.

**Request**

```jsx
{
	claim: {}
}
```

**Log**

```jsx
[
	{key: "action", value: "claim"},
	{key: "sender", value: "{address}"},
	{key: "claim_amount", value: "{amount}"}
]
```

### Exit

- unlock all DP tokens locked with this contract, and claim all accumulated rewards.

**Request**

```jsx
{
	exit: {}
}
```

**Log**

```jsx
[
	{key: "action", value: "exit"},
	{key: "sender", value: "{address}"},
	{key: "claim_amount", value: "{amount}"},
	{key: "withdraw_amount", value: "{amount}"}
]
```

## QueryMsg

### Config

- returns configuration data of this contract.

**Request**

```jsx
{
	config: {}
}
```

**Response**

- `owner`: owner of the contract
- `share_token`: (for MINE sales) DP Token contract address
- `reward_token`: (for MINE sales) MINE Token contract address; may be replaced with any other token address being sold with this lockup pool.
- `start_time`: distribution start time
- `cliff_time`: timestamp of which underlying `reward_token`s start being distributed
- `finish_time`: timestamp of which underlying `reward_token`s stop being distributed
- `reward_rate`: number of reward tokens distributed per second

```jsx
{
	owner: "{address}", // AccAddress
	share_token: "{address}", // AccAddress
	reward_token: "{address}", // AccAddress
	start_time: 1622870382, // uint64, blocktime
	cliff_time: 1622870382, // uint64, blocktime
	finish_time: 1622870382, // uint64, blocktime
	reward_rate: 1.32993, // Decimal256	
}
```

### Reward

- returns reward-related configuration data (state) of this contract.

**Request**

```jsx
{
	reward: {}
}
```

**Response**

- `total_deposit`: total deposit amount of this contract
- `last_update_time`: timestamp of which rewards data was last updated

```jsx
{
	total_deposit: 1000000000, // Uint256, 6 decimal
	last_update_time: 1622870382 // uint64, blocktime
}
```

### BalanceOf

- returns deposit amount of a specific wallet

**Request**

- `owner`: wallet address to query deposited coins

```jsx
{
	balance_of: {
		owner: "{address}" // AccAddress
	}
}
```

**Response**

- `amount`: deposited amount (denominated in UST)

```jsx
{
	amount: 1000000000 // Uint256 - 6 decimals
}
```

### ClaimableReward

- returns total claimable rewards for a specific wallet at a specified timestamp.

**Request**

- `owner`: wallet address to query outstanding rewards.
- `timestamp`: [OPTIONAL] timestamp to query reward balances from.

```jsx
{
	claimable_reward: {
		owner: "{address}", // AccAddress
		timestamp: 1622870382 // uint64, blocktime
	}
}
```

**Response**

- `amount`: claimable reward amount (denominated in UST)

```jsx
{
	amount: 100000000 // Uint256, 6 decimal
}
```