import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { Rollup, Yeeter } from '@aztec/l1-contracts';
import { beforeAll, describe, expect, it } from '@jest/globals';
import { createPublicClient, hexToBytes, http } from 'viem';
import { localhost } from 'viem/chains';
import { Archiver, INITIAL_BLOCK_NUM, L2Block, mockRandomL2Block } from '../src/index.js';

// Accounts 4 and 5 of Anvil default startup with mnemonic: 'test test test test test test test test test test test junk'
const sequencerPK = '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a';
const deployerPK = '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba';
const anvilHost = process.env.ANVIL_HOST ?? 'http://127.0.0.1:8545';

describe('Archiver integration', () => {
  let rollup: Rollup;
  let yeeter: Yeeter;
  let l2Block: L2Block;
  let l2Proof: Buffer;
  let archiver: Archiver;

  beforeAll(async () => {
    ({ rollup, yeeter } = await deployContracts());

    l2Block = mockRandomL2Block(INITIAL_BLOCK_NUM);
    l2Proof = Buffer.alloc(0);

    const publicClient = createPublicClient({
      chain: localhost,
      transport: http(anvilHost),
    });

    archiver = new Archiver(publicClient, rollup.address, yeeter.address);
  });

  it('reads l2block from archiver initial sync', async () => {
    await rollup.methods.process(l2Proof, l2Block.encode()).send({ gas: 3e6 }).getReceipt(true, 0);
        
    await archiver.start();    
    const blocks = await archiver.getL2Blocks(0, 1);
    await archiver.stop();

    expect(blocks).toHaveLength(1);
    expect(blocks[0]).toEqual(l2Block);
  });
});

async function deployContracts() {
  // Set up client
  const provider = WalletProvider.fromHost(anvilHost);
  provider.addAccount(Buffer.from(hexToBytes(deployerPK)));
  provider.addAccount(Buffer.from(hexToBytes(sequencerPK)));
  const [sequencer, deployer] = provider.getAccounts();
  const ethRpc = new EthereumRpc(provider);

  // Deploy contracts
  const deployedRollup = new Rollup(ethRpc, undefined, { from: deployer, gas: 1e6 });
  await deployedRollup.deploy().send().getReceipt();

  const deployedYeeter = new Yeeter(ethRpc, undefined, { from: deployer, gas: 1e6 });
  await deployedYeeter.deploy().send().getReceipt();

  // Create new instance so we can attach the sequencer as sender
  const rollup = new Rollup(ethRpc, deployedRollup.address, { from: sequencer });
  const yeeter = new Yeeter(ethRpc, deployedYeeter.address, { from: sequencer });

  return { rollup, yeeter, deployer, sequencer, ethRpc };
}