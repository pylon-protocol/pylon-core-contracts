import { program } from "commander";
program.version("0.0.1");
program.option("-n, --network <type>", "network type to deploy", "local");
program.option("-s, --source <type>", "source directory", "../artifacts");
program.parse();
const { source, network: networkType } = program.opts();

import {
  BlockTxBroadcastResult,
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

import { Network } from "./network.config";

const network = Network[networkType];
const lcdClient = new LCDClient({
  URL: network.URL,
  chainID: network.chainID,
  gasAdjustment: 2,
  gasPrices: { uluna: 0.15 },
});

const key = new MnemonicKey({ mnemonic: network.accounts.mnemonic });
const wallet = new Wallet(lcdClient, key);

const sleep = async (time: number) =>
  new Promise((resolve) => setInterval(resolve, time));

function extOf(filename: string): string {
  if (/[.]/.exec(filename)) {
    const ext = /[^.]+$/.exec(filename)?.toString();
    if (ext) {
      return ext;
    }
  }
  throw new Error("failed to fetch extension from filename");
}

async function main(): Promise<void> {
  console.log(lcdClient.config);

  const balance = await lcdClient.bank.balance(wallet.key.accAddress);
  console.log(balance);

  const fileNames = fs.readdirSync(source).filter((v) => extOf(v) == "wasm");

  const accInfo = await lcdClient.auth.accountInfo(wallet.key.accAddress);

  let sequence = accInfo.sequence;
  const codeIds: { [contract: string]: string } = {};
  for (const fileName of fileNames) {
    console.log(`reading ${fileName}`);
    const file = fs.readFileSync(path.join(source, fileName));

    let result: BlockTxBroadcastResult;
    for (;;) {
      const tx = await wallet.createAndSignTx({
        msgs: [
          new MsgStoreCode(wallet.key.accAddress, file.toString("base64")),
        ],
        sequence: sequence,
      });
      sequence += 1;

      result = await lcdClient.tx.broadcast(tx);
      if (isTxError(result)) {
        await sleep(1000);
        console.log(result.raw_log);
        ({ sequence } = await lcdClient.auth.accountInfo(
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
        txInfo = await lcdClient.tx.txInfo(result.txhash);
        break;
      } catch (e) {
        console.error(e);
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
  }

  fs.writeFileSync(
    `../code_id_${networkType}.json`,
    JSON.stringify(codeIds, null, 2)
  );
  return;
}

main().catch(console.error);
