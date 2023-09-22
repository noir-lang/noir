import { HistoricBlockData } from './historic_block_data.js';

describe('HistoricBlockData', () => {
  it('serialises to buffer and back', () => {
    const historicBlockData = HistoricBlockData.random();
    const serialised = historicBlockData.toBuffer();
    const deserialised = HistoricBlockData.fromBuffer(serialised);
    expect(deserialised).toEqual(historicBlockData);
  });

  it('serialises to string and back', () => {
    const historicBlockData = HistoricBlockData.random();
    const serialised = historicBlockData.toString();
    const deserialised = HistoricBlockData.fromString(serialised);
    expect(deserialised).toEqual(historicBlockData);
  });
});
