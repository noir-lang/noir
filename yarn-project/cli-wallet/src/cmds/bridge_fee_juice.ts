import { createCompatibleClient } from '@aztec/aztec.js';
import { FeeJuicePortalManager, prettyPrintJSON } from '@aztec/cli/utils';
import { createEthereumChain, createL1Clients } from '@aztec/ethereum';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

export async function bridgeL1FeeJuice(
  amount: bigint,
  recipient: AztecAddress,
  rpcUrl: string,
  l1RpcUrl: string,
  chainId: number,
  privateKey: string | undefined,
  mnemonic: string,
  mint: boolean,
  json: boolean,
  wait: boolean,
  interval = 60_000,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  // Prepare L1 client
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const { publicClient, walletClient } = createL1Clients(chain.rpcUrl, privateKey ?? mnemonic, chain.chainInfo);

  // Prepare L2 client
  const client = await createCompatibleClient(rpcUrl, debugLogger);

  const {
    protocolContractAddresses: { feeJuice: feeJuiceAddress },
  } = await client.getPXEInfo();

  // Setup portal manager
  const portal = await FeeJuicePortalManager.new(client, publicClient, walletClient, debugLogger);
  const { claimAmount, claimSecret, messageHash } = await portal.bridgeTokensPublic(recipient, amount, mint);

  if (json) {
    const out = {
      claimAmount,
      claimSecret,
    };
    log(prettyPrintJSON(out));
  } else {
    if (mint) {
      log(`Minted ${claimAmount} fee juice on L1 and pushed to L2 portal`);
    } else {
      log(`Bridged ${claimAmount} fee juice to L2 portal`);
    }
    log(`claimAmount=${claimAmount},claimSecret=${claimSecret},messageHash=${messageHash}\n`);
    log(`Note: You need to wait for two L2 blocks before pulling them from the L2 side`);
    if (wait) {
      log(
        `This command will now continually poll every ${
          interval / 1000
        }s for the inclusion of the newly created L1 to L2 message`,
      );
    }
  }

  if (wait) {
    const delayedCheck = (delay: number) => {
      return new Promise(resolve => {
        setTimeout(async () => {
          const witness = await client.getL1ToL2MembershipWitness(
            feeJuiceAddress,
            Fr.fromString(messageHash),
            claimSecret,
          );
          resolve(witness);
        }, delay);
      });
    };

    let witness;

    while (!witness) {
      witness = await delayedCheck(interval);
      if (!witness) {
        log(`No L1 to L2 message found yet, checking again in ${interval / 1000}s`);
      }
    }
  }

  return claimSecret;
}
