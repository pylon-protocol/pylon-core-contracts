import {Accounts} from "./local.config";

export const Network: { [network: string]: any } = {
    local: {
        URL: "http://localhost:1317",
        chainID: "localterra",
        ...Accounts.local,
    },
    bombay: {
        URL: "https://bombay-lcd.terra.dev",
        chainID: "bombay-12",
        ...Accounts.bombay,
    },
    columbus: {
        URL: "https://lcd.terra.dev",
        chainID: "columbus-5",
        ...Accounts.columbus,
    },
};
