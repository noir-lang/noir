import { type EthAddress } from '@aztec/circuits.js';
import { type LogFn } from '@aztec/foundation/log';

export async function dripFaucet(faucetUrl: string, asset: string, account: EthAddress, log: LogFn): Promise<void> {
  const url = new URL(`${faucetUrl}/drip/${account.toString()}`);
  url.searchParams.set('asset', asset);
  const res = await fetch(url);
  if (res.status === 200) {
    log(`Dripped ${asset} for ${account.toString()}`);
  } else if (res.status === 429) {
    log(`Rate limited when dripping ${asset} for ${account.toString()}`);
  } else {
    log(`Failed to drip ${asset} for ${account.toString()}`);
  }
}
