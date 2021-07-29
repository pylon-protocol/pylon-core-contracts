# Exchange Rate

## ExecuteMsg

Administrative calls only. Refer to contract code and comments for details. 

## QueryMsg

### ExchangeRateOf

- Query the virtual exchange rate of a specified token.

**Request**

- `token`: Target token address to query calculated virtual exchange rate.
- `blocktime`: [OPTIONAL] target blocktime to query calculated virtual exchange rate.

```json
{
	exchange_rate_of: {
		token: "{address}", // AccAddress
		blocktime: 1622872015 // uint64, blocktime
	}
}
```

**Response**

- `exchange_rate`

```json
{
	exchange_rate: 1.32345432 // Decimal256
}
```