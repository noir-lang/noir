import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, Contract, ContractDeployer, CurveType, SignerType, TxStatus } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { DebugLogger } from '@aztec/foundation/log';
import { ChildAbi, EcdsaAccountContractAbi, SchnorrAccountContractAbi } from '@aztec/noir-contracts/examples';

import { toBigInt } from '@aztec/foundation/serialize';
import { setup } from './utils.js';
import { privateKey2 } from './fixtures.js';

describe('e2e_account_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let logger: DebugLogger;

  let schnorrAccountContract: Contract;
  let ecdsaAccountContract: Contract;
  let child: Contract;

  const deployContract = async (abi: ContractAbi) => {
    logger(`Deploying L2 contract ${abi.name}...`);
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const deployMethod = deployer.deploy();
    const tx = deployMethod.send();
    await tx.isMined(0, 0.1);

    return { tx, partialContractAddress: deployMethod.partialContractAddress! };
  };

  const deployL2Contracts = async (
    schnorrCurve = CurveType.GRUMPKIN,
    schnorrSigner = SignerType.SCHNORR,
    ecdsaCurve = CurveType.SECP256K1,
    ecdsaSigner = SignerType.ECDSA,
  ) => {
    logger('Deploying Schnorr based Account contract');
    const schnorrDeploymentTx = await deployContract(SchnorrAccountContractAbi);
    const ecdsaDeploymentTx = await deployContract(EcdsaAccountContractAbi);

    const schnorrReceipt = await schnorrDeploymentTx.tx.getReceipt();
    const ecdsaReceipt = await ecdsaDeploymentTx.tx.getReceipt();

    schnorrAccountContract = new Contract(schnorrReceipt.contractAddress!, SchnorrAccountContractAbi, aztecRpcServer);
    ecdsaAccountContract = new Contract(ecdsaReceipt.contractAddress!, EcdsaAccountContractAbi, aztecRpcServer);
    logger(`L2 contract ${SchnorrAccountContractAbi.name} deployed at ${schnorrAccountContract.address}`);
    logger(`L2 contract ${EcdsaAccountContractAbi.name} deployed at ${schnorrAccountContract.address}`);

    await aztecRpcServer.registerSmartAccount(
      privateKey2,
      schnorrAccountContract.address,
      schnorrDeploymentTx.partialContractAddress!,
      schnorrCurve,
      schnorrSigner,
      SchnorrAccountContractAbi,
    );

    await aztecRpcServer.registerSmartAccount(
      privateKey2,
      ecdsaAccountContract.address,
      ecdsaDeploymentTx.partialContractAddress!,
      ecdsaCurve,
      ecdsaSigner,
      EcdsaAccountContractAbi,
    );

    const childDeployTx = await deployContract(ChildAbi);
    const childReceipt = await childDeployTx.tx.getReceipt();
    child = new Contract(childReceipt.contractAddress!, ChildAbi, aztecRpcServer);
  };

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, logger } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode.stop();
    await aztecRpcServer.stop();
  });

  it('calls a private function', async () => {
    await deployL2Contracts();
    logger('Calling private function...');
    const tx1 = child.methods.value(42).send({ from: schnorrAccountContract.address });
    const tx2 = child.methods.value(53).send({ from: ecdsaAccountContract.address });

    const txs = [tx1, tx2];

    await Promise.all(txs.map(tx => tx.isMined(0, 0.1)));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));

    expect(receipts[0].status).toBe(TxStatus.MINED);
    expect(receipts[1].status).toBe(TxStatus.MINED);
  }, 60_000);

  it('calls a public function', async () => {
    await deployL2Contracts();
    logger('Calling public function...');
    const tx1 = child.methods.pubStoreValue(42).send({ from: schnorrAccountContract.address });
    const tx2 = child.methods.pubStoreValue(53).send({ from: ecdsaAccountContract.address });

    const txs = [tx1, tx2];

    await Promise.all(txs.map(tx => tx.isMined(0, 0.1)));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));

    expect(receipts[0].status).toBe(TxStatus.MINED);
    expect(receipts[1].status).toBe(TxStatus.MINED);
    // The contract accumulates the values so the expected value is 95
    expect(toBigInt((await aztecNode.getStorageAt(child.address, 1n))!)).toEqual(95n);
  }, 60_000);

  it('fails to execute function with invalid schnorr signature', async () => {
    logger('Registering ecdsa signer against schnorr account contract');
    // Set the incorrect signer for schnorr
    await deployL2Contracts(CurveType.GRUMPKIN, SignerType.ECDSA, CurveType.SECP256K1, SignerType.ECDSA);
    await expect(child.methods.value(42).create({ from: schnorrAccountContract.address })).rejects.toMatch(
      /could not satisfy all constraints/,
    );
  }, 60_000);

  it('fails to execute function with invalid ecdsa signature', async () => {
    logger('Registering schnorr signer against ecdsa account contract');
    // Set the incorrect signer for ecdsa
    await deployL2Contracts(CurveType.GRUMPKIN, SignerType.SCHNORR, CurveType.SECP256K1, SignerType.SCHNORR);
    await expect(child.methods.value(42).create({ from: ecdsaAccountContract.address })).rejects.toMatch(
      /could not satisfy all constraints/,
    );
  }, 60_000);
});
