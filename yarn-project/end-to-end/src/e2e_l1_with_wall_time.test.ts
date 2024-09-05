import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type DebugLogger, Fr, GrumpkinScalar, type PXE, type SentTx, TxStatus } from '@aztec/aztec.js';
import { ETHEREUM_SLOT_DURATION, EthAddress } from '@aztec/circuits.js';
import { type PXEService } from '@aztec/pxe';

import { privateKeyToAccount } from 'viem/accounts';

import { getPrivateKeyFromIndex, setup } from './fixtures/utils.js';

describe('e2e_l1_with_wall_time', () => {
  let logger: DebugLogger;
  let teardown: () => Promise<void>;
  let pxe: PXE;

  beforeEach(async () => {
    const account = privateKeyToAccount(`0x${getPrivateKeyFromIndex(0)!.toString('hex')}`);
    const initialValidators = [EthAddress.fromString(account.address)];

    ({ teardown, logger, pxe } = await setup(0, { initialValidators, l1BlockTime: ETHEREUM_SLOT_DURATION, salt: 420 }));
  });

  afterEach(() => teardown());

  it('should produce blocks with a bunch of transactions', async () => {
    for (let i = 0; i < 4; i++) {
      const txs = await submitTxsTo(pxe as PXEService, 8);
      await Promise.all(
        txs.map(async (tx, j) => {
          logger.info(`Waiting for tx ${i}-${j}: ${await tx.getTxHash()} to be mined`);
          return tx.wait();
        }),
      );
    }
  });

  // submits a set of transactions to the provided Private eXecution Environment (PXE)
  const submitTxsTo = async (pxe: PXEService, numTxs: number) => {
    const txs: SentTx[] = [];
    for (let i = 0; i < numTxs; i++) {
      const accountManager = getSchnorrAccount(pxe, Fr.random(), GrumpkinScalar.random(), Fr.random());
      const deployMethod = await accountManager.getDeployMethod();
      await deployMethod.create({
        contractAddressSalt: accountManager.salt,
        skipClassRegistration: true,
        skipPublicDeployment: true,
        universalDeploy: true,
      });
      await deployMethod.prove({});
      const tx = deployMethod.send();

      const txHash = await tx.getTxHash();

      logger.info(`Tx sent with hash ${txHash}`);
      const receipt = await tx.getReceipt();
      expect(receipt).toEqual(
        expect.objectContaining({
          status: TxStatus.PENDING,
          error: '',
        }),
      );
      logger.info(`Receipt received for ${txHash}`);
      txs.push(tx);
    }
    return txs;
  };
});
