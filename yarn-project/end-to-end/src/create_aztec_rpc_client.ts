import { AztecNode } from '@aztec/aztec-node';
import { createAztecRPCServer } from '@aztec/aztec.js';
import { createAztecNode } from './create_aztec_node.js';

export async function createAztecRPCClient(numberOfAccounts = 1, aztecNode?: AztecNode) {
  const node = aztecNode || (await createAztecNode());
  const arc = await createAztecRPCServer(node);

  for (let i = 0; i < numberOfAccounts; ++i) {
    await arc.addAccount();
  }

  return arc;
}
