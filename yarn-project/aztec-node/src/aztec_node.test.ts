import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { AccumulatedTxData, Tx } from '@aztec/p2p';
import { AztecNode } from './index.js';

const ETHEREUM_HOST = 'http://localhost:8545/';
const ROLLUP_ADDRESS = '0x5FbDB2315678afecb367f032d93F642f64180aa3';
const YEETER_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512';

describe('AztecNode', () => {
  it('should start and stop all services', async () => {
    const node = new AztecNode();
    await node.init(ETHEREUM_HOST, EthAddress.fromString(ROLLUP_ADDRESS), EthAddress.fromString(YEETER_ADDRESS));
    const isReady = await node.isReady();
    expect(isReady).toBeTruthy();
    await node.stop();
  });

  it('should accept a transaction', async () => {
    const node = new AztecNode();
    await node.init(ETHEREUM_HOST, EthAddress.fromString(ROLLUP_ADDRESS), EthAddress.fromString(YEETER_ADDRESS));
    const isReady = await node.isReady();
    expect(isReady).toBeTruthy();
    const tx: Tx = new Tx(AccumulatedTxData.random());
    await node.sendTx(tx);
    const txs = await node.getTxs();
    expect(txs.length).toBe(1);
    expect(txs[0].txId).toEqual(tx.txId);
  });
});
