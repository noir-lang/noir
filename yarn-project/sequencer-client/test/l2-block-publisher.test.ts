import { L2Block, mockRandomL2Block } from '@aztec/archiver';
import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { Rollup } from '@aztec/l1-contracts';
import { beforeAll, describe, expect, it } from '@jest/globals';
import { EthereumjsTxSender } from '../src/publisher/ethereumjs-tx-sender.js';
import { L2BlockPublisher } from '../src/publisher/l2-block-publisher.js';
import { hexStringToBuffer } from '../src/utils.js';

// Accounts 4 and 5 of Anvil default startup with mnemonic: 'test test test test test test test test test test test junk'
const sequencerPK = '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a';
const deployerPK = '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba';
const anvilHost = process.env.ANVIL_HOST ?? 'http://127.0.0.1:8545';

describe('L2BlockPublisher integration', () => {
  let rollup: Rollup;
  let ethRpc: EthereumRpc;
  let publisher: L2BlockPublisher;
  let l2Block: L2Block;
  let l2Proof: Buffer;

  beforeAll(async () => {
    ({ ethRpc, rollup } = await deployRollup());

    l2Block = mockRandomL2Block(42);
    l2Proof = Buffer.alloc(0);

    publisher = new L2BlockPublisher(
      new EthereumjsTxSender({ 
        ethereumHost: anvilHost, 
        requiredConfirmations: 1, 
        rollupContract: rollup.address, 
        sequencerPrivateKey: hexStringToBuffer(sequencerPK)
      }),
      {
        sleepTimeMs: 100
      }
    );
  });

  it('publishes l2 block data to l1 rollup contract', async () => {
    const blockNumber = await ethRpc.blockNumber();
    await publisher.processL2Block(l2Block);

    const logs = await rollup.getLogs('L2BlockProcessed', { fromBlock: blockNumber });
    expect(logs).toHaveLength(1);
    expect(logs[0].args.blockNum).toEqual(42n);

    const tx = await ethRpc.getTransactionByHash(logs[0].transactionHash!);
    const expectedData = rollup.methods.process(l2Proof, l2Block.encode()).encodeABI();
    expect(tx.input).toEqual(expectedData);
  });
});


async function deployRollup() {
  // Set up client
  const provider = WalletProvider.fromHost(anvilHost);
  provider.addAccount(hexStringToBuffer(deployerPK));
  provider.addAccount(hexStringToBuffer(sequencerPK));
  const [sequencer, deployer] = provider.getAccounts();
  const ethRpc = new EthereumRpc(provider);

  // Deploy rollup contract
  const deployedRollup = new Rollup(ethRpc, undefined, { from: deployer, gas: 1e6 });
  await deployedRollup.deploy().send().getReceipt();

  // Create new instance so we can attach the sequencer as sender
  const rollup = new Rollup(ethRpc, deployedRollup.address, { from: sequencer });

  return { rollup, deployer, sequencer, ethRpc };
}