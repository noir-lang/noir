import { HistoricBlockData } from './historic_block_data.js';

describe('HistoricBlockData', () => {
  it('serializes to buffer and back', () => {
    const historicBlockData = HistoricBlockData.random();
    const serialized = historicBlockData.toBuffer();
    const deserialized = HistoricBlockData.fromBuffer(serialized);
    expect(deserialized).toEqual(historicBlockData);
  });

  it('serializes to string and back', () => {
    const historicBlockData = HistoricBlockData.random();
    const serialized = historicBlockData.toString();
    const deserialized = HistoricBlockData.fromString(serialized);
    expect(deserialized).toEqual(historicBlockData);
  });
});
