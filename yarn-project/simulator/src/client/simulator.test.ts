import { type AztecNode, CompleteAddress, Note } from '@aztec/circuit-types';
import { GeneratorIndex, computeAppNullifierSecretKey, deriveKeys } from '@aztec/circuits.js';
import {
  computeInnerNoteHash,
  computeNoteContentHash,
  computeUniqueNoteHash,
  siloNoteHash,
} from '@aztec/circuits.js/hash';
import {
  ABIParameterVisibility,
  type FunctionArtifactWithDebugMetadata,
  getFunctionArtifact,
} from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { poseidon2Hash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

describe('Simulator', () => {
  let oracle: MockProxy<DBOracle>;
  let node: MockProxy<AztecNode>;

  let simulator: AcirSimulator;
  let owner: AztecAddress;
  let contractAddress: AztecAddress;
  let appNullifierSecretKey: Fr;

  beforeEach(() => {
    const ownerSk = Fr.fromString('2dcc5485a58316776299be08c78fa3788a1a7961ae30dc747fb1be17692a8d32');
    const allOwnerKeys = deriveKeys(ownerSk);

    const ownerMasterNullifierPublicKey = allOwnerKeys.masterNullifierPublicKey;
    const ownerMasterNullifierSecretKey = allOwnerKeys.masterNullifierSecretKey;

    contractAddress = AztecAddress.random();

    const ownerPartialAddress = Fr.random();
    const ownerCompleteAddress = CompleteAddress.fromSecretKeyAndPartialAddress(ownerSk, ownerPartialAddress);
    owner = ownerCompleteAddress.address;

    appNullifierSecretKey = computeAppNullifierSecretKey(ownerMasterNullifierSecretKey, contractAddress);

    oracle = mock<DBOracle>();
    node = mock<AztecNode>();
    oracle.getNullifierKeys.mockResolvedValue({
      masterNullifierPublicKey: ownerMasterNullifierPublicKey,
      appNullifierSecretKey,
    });
    oracle.getCompleteAddress.mockResolvedValue(ownerCompleteAddress);

    simulator = new AcirSimulator(oracle, node);
  });

  describe('computeNoteHashAndNullifier', () => {
    const artifact = getFunctionArtifact(TokenContractArtifact, 'compute_note_hash_and_nullifier');
    const nonce = Fr.random();
    const storageSlot = TokenContractArtifact.storageLayout['balances'].slot;
    const noteTypeId = TokenContractArtifact.notes['TokenNote'].id;

    const createNote = (amount = 123n) => new Note([new Fr(amount), owner.toField(), Fr.random()]);

    it('should compute note hashes and nullifier', async () => {
      oracle.getFunctionArtifactByName.mockResolvedValue(artifact);

      const note = createNote();
      const tokenNoteHash = computeNoteContentHash(note.items);
      const innerNoteHash = computeInnerNoteHash(storageSlot, tokenNoteHash);
      const siloedNoteHash = siloNoteHash(contractAddress, innerNoteHash);
      const uniqueSiloedNoteHash = computeUniqueNoteHash(nonce, siloedNoteHash);
      const innerNullifier = poseidon2Hash([
        uniqueSiloedNoteHash,
        appNullifierSecretKey,
        GeneratorIndex.NOTE_NULLIFIER,
      ]);

      const result = await simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, noteTypeId, note);

      expect(result).toEqual({
        innerNoteHash,
        siloedNoteHash,
        uniqueSiloedNoteHash,
        innerNullifier,
      });
    });

    it('throw if the contract does not implement "compute_note_hash_and_nullifier"', async () => {
      oracle.getFunctionArtifactByName.mockResolvedValue(undefined);

      const note = createNote();
      await expect(
        simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, noteTypeId, note),
      ).rejects.toThrow(/Mandatory implementation of "compute_note_hash_and_nullifier" missing/);
    });

    it('throw if "compute_note_hash_and_nullifier" has the wrong number of parameters', async () => {
      const note = createNote();

      const modifiedArtifact: FunctionArtifactWithDebugMetadata = {
        ...artifact,
        parameters: artifact.parameters.slice(1),
      };
      oracle.getFunctionArtifactByName.mockResolvedValue(modifiedArtifact);

      await expect(
        simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, noteTypeId, note),
      ).rejects.toThrow(
        new RegExp(
          `Expected 5 parameters in mandatory implementation of "compute_note_hash_and_nullifier", but found 4 in noir contract ${contractAddress}.`,
        ),
      );
    });

    it('throw if a note has more fields than "compute_note_hash_and_nullifier" can process', async () => {
      const note = createNote();
      const wrongPreimageLength = note.length - 1;

      const modifiedArtifact: FunctionArtifactWithDebugMetadata = {
        ...artifact,
        parameters: [
          ...artifact.parameters.slice(0, -1),
          {
            name: 'note',
            type: {
              kind: 'array',
              length: wrongPreimageLength,
              type: {
                kind: 'field',
              },
            },
            visibility: ABIParameterVisibility.SECRET,
          },
        ],
      };
      oracle.getFunctionArtifactByName.mockResolvedValue(modifiedArtifact);

      await expect(
        simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, noteTypeId, note),
      ).rejects.toThrow(
        new RegExp(`"compute_note_hash_and_nullifier" can only handle a maximum of ${wrongPreimageLength} fields`),
      );
    });
  });
});
