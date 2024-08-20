import { type AztecAddress, type Fr, createCompatibleClient } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

export async function getL1ToL2MessageWitness(
  rpcUrl: string,
  contractAddress: AztecAddress,
  messageHash: Fr,
  secret: Fr,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const messageWitness = await client.getL1ToL2MembershipWitness(contractAddress, messageHash, secret);

  log(
    messageWitness === undefined
      ? `
    L1 to L2 Message not found.
    `
      : `
    L1 to L2 message index: ${messageWitness[0]}
    L1 to L2 message sibling path: ${messageWitness[1]}
    `,
  );
}
