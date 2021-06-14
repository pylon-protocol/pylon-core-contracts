import { Accounts } from "./local.config";

export const Network: { [network: string]: any } = {
  local: {
    URL: "http://localhost:1317",
    chainID: "localterra",
    ...Accounts.local,
  },
  tequila: {
    URL: "https://tequila-lcd.terra.dev",
    chainID: "tequila-0004",
    ...Accounts.tequila,
  },
  columbus: {
    URL: "https://lcd.terra.dev",
    chainID: "columbus-4",
    ...Accounts.columbus,
  },
};
