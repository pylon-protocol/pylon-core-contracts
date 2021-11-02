import { program } from "commander";

program.version("0.0.1");
program.option("-n, --network <type>", "network type to deploy", "local");
program.option("-s, --source <type>", "source file");
program.option("-d --directory <type>", "source directory");
program.parse();
const { source, directory, network: networkType } = program.opts();

import {
  BlockTxBroadcastResult,
  Coins,
  isTxError,
  LCDClient,
  MnemonicKey,
  MsgStoreCode,
  TxInfo,
  Wallet,
} from "@terra-money/terra.js";
import * as fs from "fs";
import * as path from "path";
import * as util from "util";
import axios from "axios";

import { Network } from "./network.config";
import { exit } from "process";

const sleep = async (time: number) =>
  new Promise((resolve) => setInterval(resolve, time));

async function fetchGasPrices(denoms: string[]): Promise<Coins> {
  const { data } = await axios.get("https://fcd.terra.dev/v1/txs/gas_prices");
  console.log(denoms.map((denom) => `${data[denom]}${denom}`).join(","));
  const gasPrices = new Coins(
    denoms.map((denom) => `${data[denom]}${denom}`).join(",")
  );
  return gasPrices;
}

function extOf(filename: string): string {
  if (/[.]/.exec(filename)) {
    const ext = /[^.]+$/.exec(filename)?.toString();
    if (ext) {
      return ext;
    }
  }
  throw new Error("failed to fetch extension from filename");
}

async function deploySource(wallet: Wallet, path: string) {
  const file = fs.readFileSync(path);

  let result: BlockTxBroadcastResult;

  const tx = await wallet.createAndSignTx({
    msgs: [new MsgStoreCode(wallet.key.accAddress, file.toString("base64"))],
  });
  result = await wallet.lcd.tx.broadcast(tx);

  let txInfo: TxInfo;
  for (;;) {
    try {
      txInfo = await wallet.lcd.tx.txInfo(result.txhash);
      break;
    } catch (e) {
      if (axios.isAxiosError(e) && e.response) {
        console.error(e.response.data.error);
      } else {
        console.error(`Unexpected error: ${e}`);
      }
      await sleep(1000);
    }
  }
  for (const log of txInfo.logs || []) {
    const events = log.events.filter((v) => v.type == "store_code");
    if (events.length == 0) {
      console.log(util.inspect(log, { depth: null }));
      throw new Error("?");
    }
    const [{ value: sender }, { value: codeId }] = events[0].attributes;

    console.log(`=> sender: ${sender}`);
    console.log(`=> codeId: ${codeId}`);
  }
}

async function deployDirectory(wallet: Wallet) {
  const fileNames = fs.readdirSync(directory).filter((v) => extOf(v) == "wasm");
  let { sequence } = await wallet.lcd.auth.accountInfo(wallet.key.accAddress);
  const codeIds: { [contract: string]: string } = {};
  for (const fileName of fileNames) {
    console.log(`reading ${fileName}`);
    const file = fs.readFileSync(path.join(directory, fileName));

    let result: BlockTxBroadcastResult;
    for (;;) {
      const tx = await wallet.createAndSignTx({
        msgs: [
          new MsgStoreCode(wallet.key.accAddress, file.toString("base64")),
        ],
        sequence: sequence,
      });
      sequence += 1;

      result = await wallet.lcd.tx.broadcast(tx);
      if (isTxError(result)) {
        await sleep(1000);
        console.log(result.raw_log);
        ({ sequence } = await wallet.lcd.auth.accountInfo(
          wallet.key.accAddress
        ));
        continue;
      }
      break;
    }

    console.log(`${fileName} ${result.txhash}`);

    let txInfo: TxInfo;
    for (;;) {
      try {
        txInfo = await wallet.lcd.tx.txInfo(result.txhash);
        break;
      } catch (e) {
        if (axios.isAxiosError(e) && e.response) {
          console.error(e.response.data.error);
        } else {
          console.error(`Unexpected error: ${e}`);
        }
        await sleep(1000);
      }
    }
    for (const log of txInfo.logs || []) {
      const events = log.events.filter((v) => v.type == "store_code");
      if (events.length == 0) {
        console.log(util.inspect(log, { depth: null }));
        throw new Error("?");
      }
      const [{ value: sender }, { value: codeId }] = events[0].attributes;

      console.log(`=> sender: ${sender}`);
      console.log(`=> codeId: ${codeId}`);

      codeIds[fileName.replace(/\.[^/.]+$/, "")] = codeId;
    }
    await sleep(3000);
  }

  fs.writeFileSync(
    `../code_id_${networkType}.json`,
    JSON.stringify(codeIds, null, 2)
  );
}

async function main(): Promise<void> {
  const network = Network[networkType];
  const client = new LCDClient({
    URL: network.URL,
    chainID: network.chainID,
    gasAdjustment: 1.5,
    gasPrices: await fetchGasPrices([
      "uusd",
      //   "uluna",
    ]),
  });

  const key = new MnemonicKey({ mnemonic: network.accounts.mnemonic });
  const wallet = new Wallet(client, key);
  console.log(client.config);

  const balance = await client.bank.balance(wallet.key.accAddress);
  console.log(wallet.key.accAddress);
  console.log(balance);

  if (directory) {
    await deployDirectory(wallet);
  } else if (source) {
    await deploySource(wallet, source);
  }

  exit(0);
}

main().catch(console.error);
