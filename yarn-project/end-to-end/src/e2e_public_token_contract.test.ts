import { AztecAddress, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { PublicTokenContract } from '@aztec/noir-contracts/types';
import { CompleteAddress, PXE, TxStatus } from '@aztec/types';

import times from 'lodash.times';

import { expectUnencryptedLogsFromLastBlockToBe, expectUnencryptedLogsInTxToBe, setup } from './fixtures/utils.js';

describe('e2e_public_token_contract', () => {
  let pxe: PXE;
  let wallet: Wallet;
  let logger: DebugLogger;
  let recipient: AztecAddress;
  let teardown: () => Promise<void>;

  let contract: PublicTokenContract;

  const deployContract = async () => {
    logger(`Deploying L2 public contract...`);
    const txReceipt = await PublicTokenContract.deploy(wallet).send().wait();
    contract = txReceipt.contract;
    logger(`L2 contract deployed at ${txReceipt.contractAddress}`);
    return { contract, txReceipt };
  };

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ teardown, pxe, accounts, wallet, logger } = await setup());
    recipient = accounts[0].address;
  }, 100_000);

  afterEach(() => teardown());

  it('should deploy a public token contract', async () => {
    const { txReceipt } = await deployContract();
    expect(txReceipt.status).toEqual(TxStatus.MINED);
  }, 30_000);

  it('should deploy a public token contract and mint tokens to a recipient', async () => {
    const mintAmount = 359n;

    await deployContract();

    const tx = contract.methods.mint(mintAmount, recipient).send();

    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);

    const balance = await contract.methods.publicBalanceOf(recipient.toField()).view({ from: recipient });
    expect(balance).toBe(mintAmount);

    await expectUnencryptedLogsInTxToBe(tx, ['Coins minted']);
  }, 45_000);

  // Regression for https://github.com/AztecProtocol/aztec-packages/issues/640
  it('should mint tokens thrice to a recipient within the same block', async () => {
    const mintAmount = 42n;

    await deployContract();

    // Assemble two mint txs sequentially (no parallel calls to circuits!) and send them simultaneously
    const methods = times(3, () => contract.methods.mint(mintAmount, recipient));
    for (const method of methods) await method.simulate();
    const txs = await Promise.all(methods.map(method => method.send()));

    // Check that all txs got mined in the same block
    await Promise.all(txs.map(tx => tx.isMined()));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
    expect(receipts.map(r => r.status)).toEqual(times(3, () => TxStatus.MINED));
    expect(receipts.map(r => r.blockNumber)).toEqual(times(3, () => receipts[0].blockNumber));

    const balance = await contract.methods.publicBalanceOf(recipient.toField()).view({ from: recipient });
    expect(balance).toBe(mintAmount * 3n);

    await expectUnencryptedLogsFromLastBlockToBe(pxe, ['Coins minted', 'Coins minted', 'Coins minted']);
  }, 60_000);
});
