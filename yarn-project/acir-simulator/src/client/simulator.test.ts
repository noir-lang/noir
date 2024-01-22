import { Note } from '@aztec/circuit-types';
import { CompleteAddress } from '@aztec/circuits.js';
import { computeUniqueCommitment, siloCommitment } from '@aztec/circuits.js/abis';
import { ABIParameterVisibility, FunctionArtifactWithDebugMetadata, getFunctionArtifact } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr, GrumpkinScalar, Point } from '@aztec/foundation/fields';
import { TokenContractArtifact } from '@aztec/noir-contracts/Token';

import { MockProxy, mock } from 'jest-mock-extended';

import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

describe('Simulator', () => {
  let oracle: MockProxy<DBOracle>;
  let simulator: AcirSimulator;
  const ownerPk = GrumpkinScalar.fromString('2dcc5485a58316776299be08c78fa3788a1a7961ae30dc747fb1be17692a8d32');
  const ownerCompleteAddress = CompleteAddress.fromPrivateKeyAndPartialAddress(ownerPk, Fr.random());
  const owner = ownerCompleteAddress.address;
  const ownerNullifierSecretKey = GrumpkinScalar.random();
  const ownerNullifierPublicKey = Point.random();

  const hashFields = (data: Fr[]) => Fr.fromBuffer(pedersenHash(data.map(f => f.toBuffer())));

  beforeEach(() => {
    oracle = mock<DBOracle>();
    oracle.getNullifierKeyPair.mockResolvedValue({
      secretKey: ownerNullifierSecretKey,
      publicKey: ownerNullifierPublicKey,
    });
    oracle.getCompleteAddress.mockResolvedValue(ownerCompleteAddress);

    simulator = new AcirSimulator(oracle);
  });

  describe('computeNoteHashAndNullifier', () => {
    const artifact = getFunctionArtifact(TokenContractArtifact, 'compute_note_hash_and_nullifier');
    const contractAddress = AztecAddress.random();
    const nonce = Fr.random();
    const storageSlot = Fr.random();

    const createNote = (amount = 123n) => new Note([new Fr(amount), owner.toField(), Fr.random()]);

    it('should compute note hashes and nullifier', async () => {
      oracle.getFunctionArtifactByName.mockResolvedValue(artifact);

      const note = createNote();
      const valueNoteHash = hashFields(note.items);
      const innerNoteHash = hashFields([storageSlot, valueNoteHash]);
      const siloedNoteHash = siloCommitment(contractAddress, innerNoteHash);
      const uniqueSiloedNoteHash = computeUniqueCommitment(nonce, siloedNoteHash);
      const innerNullifier = hashFields([
        uniqueSiloedNoteHash,
        ownerNullifierSecretKey.low,
        ownerNullifierSecretKey.high,
      ]);

      const result = await simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, note);

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
        simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, note),
      ).rejects.toThrowError(/Mandatory implementation of "compute_note_hash_and_nullifier" missing/);
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
        simulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, note),
      ).rejects.toThrowError(
        new RegExp(`"compute_note_hash_and_nullifier" can only handle a maximum of ${wrongPreimageLength} fields`),
      );
    });
  });
});
