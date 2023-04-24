import { PUBLIC_DATA_TREE_HEIGHT, makeEmptyProof } from '@aztec/circuits.js';
import { makeEthAddress, makeKernelPublicInputs, makePublicCircuitPublicInputs } from '@aztec/circuits.js/factories';
import { EthAddress } from '@aztec/foundation';
import { SiblingPath } from '@aztec/merkle-tree';
import { MerkleTreeOperations, TreeInfo } from '@aztec/world-state';
import { MockProxy, mock } from 'jest-mock-extended';
import pick from 'lodash.pick';
import times from 'lodash.times';
import { makePrivateTx, makePublicTx } from '../index.js';
import { Proof, PublicProver } from '../prover/index.js';
import { PublicCircuitSimulator, PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ContractDataSource, PublicProcessor } from './public_processor.js';

describe('public_processor', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let publicCircuit: MockProxy<PublicCircuitSimulator>;
  let publicKernel: MockProxy<PublicKernelCircuitSimulator>;
  let publicProver: MockProxy<PublicProver>;
  let contractDataSource: MockProxy<ContractDataSource>;

  let proof: Proof;
  let root: Buffer;
  let bytecode: Buffer;
  let portalAddress: EthAddress;

  let processor: PublicProcessor;

  beforeEach(() => {
    db = mock<MerkleTreeOperations>();
    publicCircuit = mock<PublicCircuitSimulator>();
    publicKernel = mock<PublicKernelCircuitSimulator>();
    publicProver = mock<PublicProver>();
    contractDataSource = mock<ContractDataSource>();

    proof = makeEmptyProof();
    root = Buffer.alloc(32, 5);
    bytecode = Buffer.alloc(128, 10);
    portalAddress = makeEthAddress();

    publicProver.getPublicCircuitProof.mockResolvedValue(proof);
    publicProver.getPublicKernelCircuitProof.mockResolvedValue(proof);
    db.getTreeInfo.mockResolvedValue({ root } as TreeInfo);
    contractDataSource.getPortalContractAddress.mockResolvedValue(portalAddress);
    contractDataSource.getPublicFunction.mockResolvedValue({ bytecode });

    processor = new PublicProcessor(db, publicCircuit, publicKernel, publicProver, contractDataSource);
  });

  it('skips non-public txs', async function () {
    const tx = makePrivateTx();
    const hash = await tx.getTxHash();
    const [processed, failed] = await processor.process([tx]);

    expect(processed).toEqual([{ hash, ...pick(tx, 'data', 'proof', 'unverifiedData') }]);
    expect(failed).toEqual([]);
  });

  it('returns failed txs without aborting entire operation', async function () {
    publicCircuit.publicCircuit.mockRejectedValue(new Error(`Failed`));

    const tx = makePublicTx();
    const [processed, failed] = await processor.process([tx]);

    expect(processed).toEqual([]);
    expect(failed).toEqual([tx]);
  });

  it('runs a public tx through the public and public kernel circuits', async function () {
    const publicCircuitOutput = makePublicCircuitPublicInputs();
    publicCircuit.publicCircuit.mockResolvedValue(publicCircuitOutput);

    const path = times(PUBLIC_DATA_TREE_HEIGHT, i => Buffer.alloc(32, i));
    db.getSiblingPath.mockResolvedValue(new SiblingPath(path));

    const output = makeKernelPublicInputs();
    publicKernel.publicKernelCircuitNoInput.mockResolvedValue(output);

    const tx = makePublicTx();
    const hash = await tx.getTxHash();
    const [processed, failed] = await processor.process([tx]);

    expect(processed).toEqual([{ hash, data: output, proof, ...pick(tx, 'txRequest') }]);
    expect(failed).toEqual([]);

    expect(publicCircuit.publicCircuit).toHaveBeenCalled();
    expect(publicKernel.publicKernelCircuitNoInput).toHaveBeenCalled();
  });
});
