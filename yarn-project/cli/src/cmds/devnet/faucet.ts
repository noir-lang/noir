import { type EthAddress } from '@aztec/circuits.js';
import { type LogFn } from '@aztec/foundation/log';

import { prettyPrintJSON } from '../../utils/commands.js';

export async function dripFaucet(
  faucetUrl: string,
  asset: string,
  account: EthAddress,
  json: boolean,
  log: LogFn,
): Promise<void> {
  const url = new URL(`${faucetUrl}/drip/${account.toString()}`);
  url.searchParams.set('asset', asset);
  const res = await fetch(url);
  if (res.status === 200) {
    if (json) {
      log(prettyPrintJSON({ ok: true }));
    } else {
      log(`Dripped ${asset} for ${account.toString()}`);
    }
  } else {
    if (json) {
      log(prettyPrintJSON({ ok: false }));
    } else if (res.status === 429) {
      log(`Rate limited when dripping ${asset} for ${account.toString()}`);
    } else {
      log(`Failed to drip ${asset} for ${account.toString()}`);
    }

    process.exit(1);
  }
}
