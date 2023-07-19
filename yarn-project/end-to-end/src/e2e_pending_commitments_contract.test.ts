import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, Fr, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { PendingCommitmentsContract } from '@aztec/noir-contracts/types';
import { TxStatus } from '@aztec/types';

import { setup } from './utils.js';

describe('e2e_pending_commitments_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let wallet: Wallet;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let contract: PendingCommitmentsContract;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup(2));
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServer?.stop();
  });

  const deployContract = async () => {
    logger(`Deploying L2 contract...`);
    const tx = PendingCommitmentsContract.deploy(aztecRpcServer).send();
    const receipt = await tx.getReceipt();
    contract = new PendingCommitmentsContract(receipt.contractAddress!, wallet);
    await tx.isMined(0, 0.1);
    await tx.getReceipt();
    logger('L2 contract deployed');
    return contract;
  };

  it('Noir function can "get" notes it just "inserted"', async () => {
    const mintAmount = 65n;
    const [owner] = accounts;

    const deployedContract = await deployContract();

    const tx = deployedContract.methods.test_insert_then_read_flat(mintAmount, owner).send({ origin: owner });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();
    expect(receipt.status).toBe(TxStatus.MINED);
  }, 60_000);

  it('Noir function can "get" notes inserted in a previous function call in same TX', async () => {
    const mintAmount = 65n;
    const [owner] = accounts;

    const deployedContract = await deployContract();

    const tx = deployedContract.methods
      .test_insert_then_read_both_in_nested_calls(
        mintAmount,
        owner,
        Fr.fromBuffer(deployedContract.methods.create_note.selector),
        Fr.fromBuffer(deployedContract.methods.get_and_check_note.selector),
      )
      .send({ origin: owner });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();
    expect(receipt.status).toBe(TxStatus.MINED);
  }, 60_000);

  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/836): test nullify & squash of pending notes
  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/892): test expected kernel failures if transient reads (or their hints) don't match
  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/836): test expected kernel failures if nullifiers (or their hints) don't match
  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/839): test creation, getting, nullifying of multiple notes
});
