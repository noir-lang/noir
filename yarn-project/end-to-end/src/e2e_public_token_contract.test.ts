import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, Fr, TxStatus } from '@aztec/aztec.js';
import { CircuitsWasm } from '@aztec/circuits.js';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { DebugLogger } from '@aztec/foundation/log';
import { PublicTokenContractAbi } from '@aztec/noir-contracts/examples';

import times from 'lodash.times';
import { pointToPublicKey, setup } from './utils.js';
import { pedersenCompressInputs } from '@aztec/circuits.js/barretenberg';

describe('e2e_public_token_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let contract: Contract;

  const deployContract = async () => {
    logger(`Deploying L2 public contract...`);
    const deployer = new ContractDeployer(PublicTokenContractAbi, aztecRpcServer);
    const tx = deployer.deploy().send();

    logger(`Tx sent with hash ${await tx.getTxHash()}`);
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, PublicTokenContractAbi, aztecRpcServer);
    await tx.isMined(0, 0.1);
    const txReceipt = await tx.getReceipt();
    logger('L2 contract deployed');
    return { contract, tx, txReceipt };
  };

  const calculateStorageSlot = async (accountIdx: number): Promise<Fr> => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(accounts[accountIdx]);
    const xCoordinate = Fr.fromBuffer(ownerPublicKey.buffer.subarray(0, 32));
    const bbWasm = await CircuitsWasm.get();
    const balancesStorageSlot = new Fr(1n); // this value is manually set in the Noir contract
    const mappingStorageSlot = new Fr(4n);

    // Based on `at` function in
    // aztec3-packages/yarn-project/noir-contracts/src/contracts/noir-aztec3/src/state_vars/storage_map.nr
    const storageSlot = Fr.fromBuffer(
      pedersenCompressInputs(
        bbWasm,
        [mappingStorageSlot, balancesStorageSlot, xCoordinate].map(f => f.toBuffer()),
      ),
    );

    return storageSlot; //.value;
  };

  const expectStorageSlot = async (accountIdx: number, expectedBalance: bigint) => {
    const storageSlot = await calculateStorageSlot(accountIdx);
    const storageValue = await aztecNode.getStorageAt(contract.address!, storageSlot.value);
    if (storageValue === undefined) {
      throw new Error(`Storage slot ${storageSlot} not found`);
    }

    const balance = toBigIntBE(storageValue);

    logger(`Account ${accountIdx} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, logger } = await setup());
  }, 30_000);

  afterEach(async () => {
    await aztecNode.stop();
    await aztecRpcServer.stop();
  });

  it('should deploy a public token contract', async () => {
    const { txReceipt } = await deployContract();
    expect(txReceipt.status).toEqual(TxStatus.MINED);
  }, 30_000);

  it('should deploy a public token contract and mint tokens to a recipient', async () => {
    const mintAmount = 359n;

    const recipientIdx = 0;

    const recipient = accounts[recipientIdx];
    const { contract: deployedContract } = await deployContract();

    const PK = await aztecRpcServer.getAccountPublicKey(recipient);

    const tx = deployedContract.methods.mint(mintAmount, pointToPublicKey(PK)).send({ from: recipient });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    await expectStorageSlot(recipientIdx, mintAmount);
  }, 45_000);

  // Regression for https://github.com/AztecProtocol/aztec-packages/issues/640
  it('should mint tokens thrice to a recipient within the same block', async () => {
    const mintAmount = 42n;
    const recipientIdx = 0;
    const recipient = accounts[recipientIdx];
    const pubKey = pointToPublicKey(await aztecRpcServer.getAccountPublicKey(recipient));
    const { contract: deployedContract } = await deployContract();

    // Assemble two mint txs sequentially (no parallel calls to circuits!) and send them simultaneously
    const methods = times(3, () => deployedContract.methods.mint(mintAmount, pubKey));
    for (const method of methods) await method.create({ from: recipient });
    const txs = await Promise.all(methods.map(method => method.send()));

    // Check that all txs got mined in the same block
    await Promise.all(txs.map(tx => tx.isMined()));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
    expect(receipts.map(r => r.status)).toEqual(times(3, () => TxStatus.MINED));
    expect(receipts.map(r => r.blockNumber)).toEqual(times(3, () => receipts[0].blockNumber));

    await expectStorageSlot(recipientIdx, mintAmount * 3n);
  }, 60_000);
});
