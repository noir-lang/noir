import { AztecNode } from '@aztec/aztec-node';
import { createAztecRPCServer } from '@aztec/aztec.js';

export async function createAztecRpcServer(numberOfAccounts = 1, aztecNode: AztecNode) {
  const arc = await createAztecRPCServer(aztecNode);

  for (let i = 0; i < numberOfAccounts; ++i) {
    await arc.addAccount();
  }

  return arc;
}
