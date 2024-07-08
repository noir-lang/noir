import { type AztecNode, CompleteAddress, Note } from '@aztec/circuit-types';
import { GeneratorIndex, KeyValidationRequest, computeAppNullifierSecretKey, deriveKeys } from '@aztec/circuits.js';
import {
  computeInnerNoteHash,
  computeNoteContentHash,
  computeUniqueNoteHash,
  siloNoteHash,
} from '@aztec/circuits.js/hash';
import { type FunctionArtifact, getFunctionArtifact } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { poseidon2Hash } from '@aztec/foundation/crypto';
import { Fr, type Point } from '@aztec/foundation/fields';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

describe('Simulator', () => {
  let oracle: MockProxy<DBOracle>;
  let node: MockProxy<AztecNode>;

  let simulator: AcirSimulator;
  let ownerMasterNullifierPublicKey: Point;
  let contractAddress: AztecAddress;
  let appNullifierSecretKey: Fr;

  beforeEach(() => {
    const ownerSk = Fr.fromString('2dcc5485a58316776299be08c78fa3788a1a7961ae30dc747fb1be17692a8d32');
    const allOwnerKeys = deriveKeys(ownerSk);

    ownerMasterNullifierPublicKey = allOwnerKeys.publicKeys.masterNullifierPublicKey;
    const ownerMasterNullifierSecretKey = allOwnerKeys.masterNullifierSecretKey;

    contractAddress = AztecAddress.random();

    const ownerPartialAddress = Fr.random();
    const ownerCompleteAddress = CompleteAddress.fromSecretKeyAndPartialAddress(ownerSk, ownerPartialAddress);

    appNullifierSecretKey = computeAppNullifierSecretKey(ownerMasterNullifierSecretKey, contractAddress);

    oracle = mock<DBOracle>();
    node = mock<AztecNode>();
    oracle.getKeyValidationRequest.mockResolvedValue(
      new KeyValidationRequest(ownerMasterNullifierPublicKey, appNullifierSecretKey),
    );
    oracle.getCompleteAddress.mockResolvedValue(ownerCompleteAddress);

    simulator = new AcirSimulator(oracle, node);
  });

  describe('computeNoteHashAndOptionallyANullifier', () => {
    const artifact = getFunctionArtifact(TokenContractArtifact, 'compute_note_hash_and_optionally_a_nullifier');
    const nonce = Fr.random();
    const storageSlot = TokenContractArtifact.storageLayout['balances'].slot;
    const noteTypeId = TokenContractArtifact.notes['TokenNote'].id;

    const createNote = (amount = 123n) => new Note([new Fr(amount), ownerMasterNullifierPublicKey.hash(), Fr.random()]);

    it('should compute note hashes and nullifier', async () => {
      oracle.getFunctionArtifactByName.mockResolvedValue(artifact);

      const note = createNote();
      const tokenNoteHash = computeNoteContentHash(note.items);
      const innerNoteHash = computeInnerNoteHash(storageSlot, tokenNoteHash);
      const uniqueNoteHash = computeUniqueNoteHash(nonce, innerNoteHash);
      const siloedNoteHash = siloNoteHash(contractAddress, uniqueNoteHash);
      const innerNullifier = poseidon2Hash([siloedNoteHash, appNullifierSecretKey, GeneratorIndex.NOTE_NULLIFIER]);

      const result = await simulator.computeNoteHashAndOptionallyANullifier(
        contractAddress,
        nonce,
        storageSlot,
        noteTypeId,
        true,
        note,
      );

      expect(result).toEqual({
        innerNoteHash,
        uniqueNoteHash,
        siloedNoteHash,
        innerNullifier,
      });
    });

    it('throw if the contract does not implement "compute_note_hash_and_optionally_a_nullifier"', async () => {
      oracle.getFunctionArtifactByName.mockResolvedValue(undefined);

      const note = createNote();
      await expect(
        simulator.computeNoteHashAndOptionallyANullifier(contractAddress, nonce, storageSlot, noteTypeId, true, note),
      ).rejects.toThrow(/Mandatory implementation of "compute_note_hash_and_optionally_a_nullifier" missing/);
    });

    it('throw if "compute_note_hash_and_optionally_a_nullifier" has the wrong number of parameters', async () => {
      const note = createNote();

      const modifiedArtifact: FunctionArtifact = {
        ...artifact,
        parameters: artifact.parameters.slice(1),
      };
      oracle.getFunctionArtifactByName.mockResolvedValue(modifiedArtifact);

      await expect(
        simulator.computeNoteHashAndOptionallyANullifier(contractAddress, nonce, storageSlot, noteTypeId, true, note),
      ).rejects.toThrow(
        new RegExp(
          `Expected 6 parameters in mandatory implementation of "compute_note_hash_and_optionally_a_nullifier", but found 5 in noir contract ${contractAddress}.`,
        ),
      );
    });

    it('throw if a note has more fields than "compute_note_hash_and_optionally_a_nullifier" can process', async () => {
      const note = createNote();
      const wrongPreimageLength = note.length - 1;

      const modifiedArtifact: FunctionArtifact = {
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
            visibility: 'private',
          },
        ],
      };
      oracle.getFunctionArtifactByName.mockResolvedValue(modifiedArtifact);

      await expect(
        simulator.computeNoteHashAndOptionallyANullifier(contractAddress, nonce, storageSlot, noteTypeId, true, note),
      ).rejects.toThrow(
        new RegExp(
          `"compute_note_hash_and_optionally_a_nullifier" can only handle a maximum of ${wrongPreimageLength} fields`,
        ),
      );
    });
  });
});
