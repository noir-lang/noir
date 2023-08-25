import { AztecAddress, FunctionData, TxContext } from '@aztec/circuits.js';
import { FunctionAbi, encodeArguments } from '@aztec/foundation/abi';
import { NodeInfo, PackedArguments, TxExecutionRequest } from '@aztec/types';

/**
 * Utility for building a TxExecutionRequest in the context of an Entrypoint.
 * @param origin - Address of the account contract sending this transaction.
 * @param entrypointMethod - Initial method called in the account contract.
 * @param args - Arguments used when calling this initial method.
 * @param callsPackedArguments - Packed arguments of nested calls (if any).
 * @param nodeInfo - Node info with chain id and version.
 * @returns A TxExecutionRequest ready to be simulated, proven, and sent.
 */
export async function buildTxExecutionRequest(
  origin: AztecAddress,
  entrypointMethod: FunctionAbi,
  args: any[],
  callsPackedArguments: PackedArguments[],
  nodeInfo: NodeInfo,
): Promise<TxExecutionRequest> {
  const packedArgs = await PackedArguments.fromArgs(encodeArguments(entrypointMethod, args));
  const { chainId, version } = nodeInfo;

  return TxExecutionRequest.from({
    argsHash: packedArgs.hash,
    origin,
    functionData: FunctionData.fromAbi(entrypointMethod),
    txContext: TxContext.empty(chainId, version),
    packedArguments: [...callsPackedArguments, packedArgs],
  });
}
