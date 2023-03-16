import { randomAppendOnlyTreeSnapshot, randomBytes, randomContractData } from './mocks.js';
import { ContractData, L2Block } from './l2_block.js';

describe('L2Block', () => {
  it('can encode a L2 block data object to buffer and back', () => {
    const newNullifiers = [randomBytes(32), randomBytes(32), randomBytes(32), randomBytes(32)];
    const newCommitments = [randomBytes(32), randomBytes(32), randomBytes(32), randomBytes(32)];
    const newContracts: Buffer[] = [randomBytes(32)];
    const newContractsData: ContractData[] = [randomContractData()];

    const l2BlockData = new L2Block(
      0,
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(newCommitments.length),
      randomAppendOnlyTreeSnapshot(newNullifiers.length),
      randomAppendOnlyTreeSnapshot(newContracts.length),
      randomAppendOnlyTreeSnapshot(1),
      randomAppendOnlyTreeSnapshot(1),
      newCommitments,
      newNullifiers,
      newContracts,
      newContractsData,
    );

    const buffer = l2BlockData.encode();
    const recovered = L2Block.decode(buffer);

    expect(recovered).toEqual(l2BlockData);
  });
});
