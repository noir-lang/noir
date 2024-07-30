import { type L1ToL2Message } from '@aztec/aztec.js';
import { type AztecAddress, Fr } from '@aztec/circuits.js';
import { type L1ContractAddresses } from '@aztec/ethereum';
import { InboxAbi } from '@aztec/l1-artifacts';

import { expect } from '@jest/globals';
import { type Hex, type PublicClient, type WalletClient, decodeEventLog, getContract } from 'viem';

export async function sendL1ToL2Message(
  message: L1ToL2Message | { recipient: AztecAddress; content: Fr; secretHash: Fr },
  ctx: {
    walletClient: WalletClient;
    publicClient: PublicClient;
    l1ContractAddresses: Pick<L1ContractAddresses, 'inboxAddress'>;
  },
) {
  const inbox = getContract({
    address: ctx.l1ContractAddresses.inboxAddress.toString(),
    abi: InboxAbi,
    client: ctx.walletClient,
  });

  const recipient = 'recipient' in message.recipient ? message.recipient.recipient : message.recipient;
  const version = 'version' in message.recipient ? message.recipient.version : 1;

  // We inject the message to Inbox
  const txHash = await inbox.write.sendL2Message(
    [
      { actor: recipient.toString() as Hex, version: BigInt(version) },
      message.content.toString() as Hex,
      message.secretHash.toString() as Hex,
    ] as const,
    {} as any,
  );

  // We check that the message was correctly injected by checking the emitted event
  const txReceipt = await ctx.publicClient.waitForTransactionReceipt({ hash: txHash });

  // Exactly 1 event should be emitted in the transaction
  expect(txReceipt.logs.length).toBe(1);

  // We decode the event and get leaf out of it
  const txLog = txReceipt.logs[0];
  const topics = decodeEventLog({
    abi: InboxAbi,
    data: txLog.data,
    topics: txLog.topics,
  });
  const receivedMsgHash = topics.args.hash;

  // We check that the leaf inserted into the subtree matches the expected message hash
  if ('hash' in message) {
    const msgHash = message.hash();
    expect(receivedMsgHash).toBe(msgHash.toString());
  }

  return Fr.fromString(receivedMsgHash);
}
