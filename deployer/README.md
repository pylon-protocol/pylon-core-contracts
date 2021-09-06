# Code deployer

code deployer deploys contract to specified network & returns JSON formatted output

## Setup

```bash
$ yarn
$ yarn start # with default settings
```

### local.config.ts

```typescript
export const Accounts: { [network: string]: any } = {
  local: {
    accounts: {
      mnemonic:
        "satisfy adjust timber high purchase tuition stool faith fine install that you unaware feed domain license impose boss human eager hat rent enjoy dawn",
    },
  },
  // define aother networks
};
```

## Commands

```
Usage: yarn start [options]

Options:
  -V, --version         output the version number
  -n, --network <type>  network type to deploy (default: "local")
  -s, --source <type>   source directory (default: "../artifacts")
  -h, --help            display help for command
```

## output (example)

```json
{
    "core_exchange_rate": "136",
    "core_pool": "137",
    "launchpad_lockup": "138",
    "launchpad_swap": "139",
    "token_airdrop": "140",
    "token_collector": "141",
    "token_community": "142",
    "token_gov": "143",
    "token_staking": "144",
    "token_vesting": "145"
}
```