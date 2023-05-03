import { EthAddress } from '@aztec/foundation/eth-address';
import { TypedData } from '@aztec/ethereum.js/eth_typed_data';

/**
 * Generates the TypedData for performing a permit.
 */
export function createPermitData(
  name: string,
  owner: EthAddress,
  spender: EthAddress,
  value: bigint,
  nonce: bigint,
  deadline: bigint,
  verifyingContract: EthAddress,
  chainId: number,
  version = '1',
): TypedData {
  const types = {
    EIP712Domain: [
      { name: 'name', type: 'string' },
      { name: 'version', type: 'string' },
      { name: 'chainId', type: 'uint256' },
      { name: 'verifyingContract', type: 'address' },
    ],
    Permit: [
      {
        name: 'owner',
        type: 'address',
      },
      {
        name: 'spender',
        type: 'address',
      },
      {
        name: 'value',
        type: 'uint256',
      },
      {
        name: 'nonce',
        type: 'uint256',
      },
      {
        name: 'deadline',
        type: 'uint256',
      },
    ],
  };
  const domain = {
    name,
    version,
    chainId: chainId,
    verifyingContract: verifyingContract.toString(),
  };
  const message = {
    owner: owner.toString(),
    spender: spender.toString(),
    value: value.toString(),
    nonce: nonce.toString(),
    deadline: deadline.toString(),
  };
  return { types, domain, message, primaryType: 'Permit' };
}
