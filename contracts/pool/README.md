# Pool

## ExecuteMsg

### Redeem // CosmWasm CW-20 `send` message

- swaps DP tokens back to UST.
- must be included with the DP token's CW-20 `send` message.
- encode relevant `json` messages in `base64` format

**Request**

```jsx
{
	redeem: {}
}

// example
{
	send: {
		//...
		msg: { // base64 format
			redeem: {}
		}
	}
}
```

**Log**

```jsx
[
	{key: "action", value: "deposit"},
	{key: "sender": value: "{address}"},
	{key: "amount": value: "{amount}"}
]
```

### Deposit

- swaps UST to this pool contract's DP token.
- A native `BankSend` message for UST (`uusd`) must be included with the same `CosmosMsg` message context (`coins`),
  otherwise transaction will be reverted.

**Request**

```jsx
{
	deposit: {}, // must contain UST in payload
}
```

**Log**

```jsx
[
	{key: "action", value: "redeem"},
	{key: "sender", value: "{address}"},
	{key: "amount", value: "{amount}"}
]
```

### ClaimReward // Only callable by contract owner

- claims any accumulated rewards from this pool.

**Request**

```jsx
{
	claim_reward: {}
}
```

**Log**

```jsx
[
	{key: "action", value: "claim_reward"},
	{key: "sender", value: "{address}"},
	{key: "amount", value: "{amount}"},
	{key: "fee", value: "{amount}"}
]
```

## QueryMsg

### DepositAmountOf

- returns the UST deposit amount of a particular wallet address.

**Request**

- `owner`: address to query UST deposit amount

```jsx
{
	deposit_amount_of: {
		owner: "{address}" // AccAddress
	}
}
```

**Response**

- `amount`: UST deposit amount

```jsx
{
	amount: 100000000 // Uint128 - 6 decimals
}
```

### TotalDepositAmount

- returns the total UST deposit amount of this pool contract.

**Request**

```jsx
{
	total_deposit_amount: {}
}
```

**Response**

- `amount`: total UST deposit amount

```jsx
{
	amount: 100000000 // Uint128 - 6 decimals
}
```

### Config

- returns configuration data of this pool contract.

**Request**

```jsx
{
	config: {}
}
```

**Response**

- `beneficiary`: yield beneficiary address
- `moneymarket`: address for the Anchor money market contract
- `stable_denom`: type of stablecoin → Anchor only supports UST for now
- `anchor_token`: aUST token address
- `dp_token`: `dp_token` token address

```jsx
{
	beneficiary: "{address}", // AccAddress
	moneymarket: "{address}", // AccAddress
	stable_denom: "uusd", // string
	anchor_token: "{address}", // AccAddress
	dp_token: "{address}", // AccAddress
}
```

### GetClaimableReward

- returns claimable rewards for the beneficiary of this pool contract, denominated in UST

**Request**

```jsx
{
	claimable_reward: {}
}
```

**Response**

- `claimable_reward`: claimable rewards, denominated in `uusd`

```jsx
{
	claimable_reward: 100000000 // Uint128 - 6 decimals
}
```
