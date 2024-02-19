import { mockTx } from '@aztec/circuit-types';
import {
  CallRequest,
  GlobalVariables,
  Header,
  MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  Proof,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { PublicExecutor } from '@aztec/simulator';
import { MerkleTreeOperations, TreeInfo } from '@aztec/world-state';

import { it } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';

import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ContractsDataSourcePublicDB, WorldStatePublicDB } from '../simulator/public_executor.js';
import { SetupPhaseManager } from './setup_phase_manager.js';

class TestSetupPhaseManager extends SetupPhaseManager {
  extractEnqueuedPublicCalls(tx: any) {
    return super.extractEnqueuedPublicCalls(tx);
  }
}

describe('setup_phase_manager', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let publicExecutor: MockProxy<PublicExecutor>;
  let publicProver: MockProxy<PublicProver>;
  let publicContractsDB: MockProxy<ContractsDataSourcePublicDB>;
  let publicWorldStateDB: MockProxy<WorldStatePublicDB>;
  let publicKernel: MockProxy<PublicKernelCircuitSimulator>;

  let proof: Proof;
  let root: Buffer;

  let phaseManager: TestSetupPhaseManager;

  beforeEach(() => {
    db = mock<MerkleTreeOperations>();
    publicExecutor = mock<PublicExecutor>();
    publicProver = mock<PublicProver>();
    publicContractsDB = mock<ContractsDataSourcePublicDB>();
    publicWorldStateDB = mock<WorldStatePublicDB>();

    proof = makeEmptyProof();
    root = Buffer.alloc(32, 5);

    publicProver.getPublicCircuitProof.mockResolvedValue(proof);
    publicProver.getPublicKernelCircuitProof.mockResolvedValue(proof);
    db.getTreeInfo.mockResolvedValue({ root } as TreeInfo);
    publicKernel = mock<PublicKernelCircuitSimulator>();
    phaseManager = new TestSetupPhaseManager(
      db,
      publicExecutor,
      publicKernel,
      publicProver,
      GlobalVariables.empty(),
      Header.empty(),
      publicContractsDB,
      publicWorldStateDB,
    );
  });

  it('does not extract non-revertible calls when none exist', function () {
    const tx = mockTx();
    tx.data.end.publicCallStack = makeTuple(MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, CallRequest.empty);
    tx.data.endNonRevertibleData.publicCallStack = makeTuple(
      MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
      CallRequest.empty,
    );
    const enqueuedNonRevertibleCalls = phaseManager.extractEnqueuedPublicCalls(tx);

    expect(enqueuedNonRevertibleCalls).toEqual([]);
  });
});
