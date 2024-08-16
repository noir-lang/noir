import { makeHeader } from '@aztec/circuits.js/testing';
import { Fr } from '@aztec/foundation/fields';

import { type PrivateKeyAccount } from 'viem';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';

import { TxHash } from '../tx/tx_hash.js';
import { BlockAttestation } from './block_attestation.js';
import { BlockProposal } from './block_proposal.js';
import { Signature } from './signature.js';

export const makeBlockProposal = async (signer?: PrivateKeyAccount): Promise<BlockProposal> => {
  signer = signer || randomSigner();

  const blockHeader = makeHeader(1);
  const archive = Fr.random();
  const txs = [0, 1, 2, 3, 4, 5].map(() => TxHash.random());
  const signature = Signature.from0xString(await signer.signMessage({ message: { raw: archive.toString() } }));

  return new BlockProposal(blockHeader, archive, txs, signature);
};

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/8028)
export const makeBlockAttestation = async (signer?: PrivateKeyAccount): Promise<BlockAttestation> => {
  signer = signer || randomSigner();

  const blockHeader = makeHeader(1);
  const archive = Fr.random();
  const signature = Signature.from0xString(await signer.signMessage({ message: { raw: archive.toString() } }));

  return new BlockAttestation(blockHeader, archive, signature);
};

export const randomSigner = (): PrivateKeyAccount => {
  const privateKey = generatePrivateKey();
  return privateKeyToAccount(privateKey);
};
