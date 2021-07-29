# Swap

`Swap` is a lockup pool that can proportionally withdraw tokens on deposit. On withdrawal, target tokens are returned to the caller after the lockup period, and penalized UST deposits before that.

Penalties are calculated based on a virtual AMM model, therefore every time someone withdraws exchange rate goes up.

Tokens are swapped 1:1 on deposit, although swapped x:1 on withdrawals.

## ExecuteMsg

### Deposit

- deposits UST to this contract.
- A native `BankSend` message for UST (`uusd`) must be included with the same `CosmosMsg` message context (`coins`), otherwise transaction will be reverted.

**Request**

```jsx
{
	deposit: {}
}
```

**Log**

```jsx
[
	{key: "action", value: "deposit"},
	{key: "sender", value: "{address}"},
	{key: "amount", value: "{amount}"}
]
```

### Withdraw

- withdraws tokens from this contract.
- withdrawing prior to the specified `finish` timestamp may result in a penalty.
- withdrawing after the specified `finish` timestamp will result in target tokens being distributed in proportion to deposited UST.

**Request**

- `amount`: coins to withdraw, denominated in UST (`uusd`)

```jsx
{
	withdraw: {
		amount: 100000000 // Uint256 - 6 decimals
	}
}
```

**Log - Penalized Deposits**

```jsx
[
	{key: "action", value: "withdraw"},
	{key: "sender", value: "{address}"},
	{key: "amount", value: "{amount}"},
	{key: "penalty", value: "{amount}"}
]
```

**Log - Retuned Reward Tokens**

```jsx
[
	{key: "action", value: "withdraw"},
	{key: "sender", value: "{address}"},
	{key: "amount", value: "{amount}"}
]
```

### Earn

- calling after the specified `finish` timestamp will result in the beneficiary receiving deposited coins.
- will revert otherwise.

**Request**

```jsx
{
	earn: {}
}
```

**Log**

```jsx
[
	{key: "action", value: "withdraw"},
	{key: "sender", value: "{address}"},
	{key: "amount", value: "{amount}"}
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

- `owner`: owner of this contract
- `beneficiary`: beneficiary address
- `start`: swap start time
- `finish`: sale finish time
- `price`: fixed sale price

```jsx
{
	owner: "{address}", // AccAddress
	beneficiary: "{address}", // AccAddress
	start: 1622873255, // uint64, blocktime
	finish: 1622883255 // uint64, blocktime
	price: 0.05 // Decimal256
}
```

### BalanceOf

- returns deposited balances of a specified wallet.

**Request**

```jsx
{
	balance_of: {
		owner: "{address}" // AccAddress
	}
}
```

**Response**

- `amount`: deposited balances, denominated in `uusd`.

```jsx
{
	amount: 10000000 // Uint256 - 6 decimals
}
```

### TotalSupply

- returns total deposited balances of this contract.

**Request**

```jsx
{
	total_supply: {}
}
```

**Response**

- `amount`: total deposited balances, denominated in `uusd`.

```jsx
{
	amount: 10000000 // Uint256 - 6 decimal
}
```

### CurrentPrice

- returns current exchange rate (penalty rate) calculated by the virtual AMM.

**Request**

```jsx
{
	current_price: {}
}
```

**Response**

- `price`: current exchange rate.

```jsx
{
	price: 1.09393 // Decimal256
}
```
