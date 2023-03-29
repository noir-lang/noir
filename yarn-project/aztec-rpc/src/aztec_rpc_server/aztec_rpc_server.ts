import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import {
  ARGS_LENGTH,
  AztecAddress,
  ContractDeploymentData,
  EthAddress,
  FunctionData,
  OldTreeRoots,
  TxContext,
  TxRequest,
  UInt8Vector,
} from '@aztec/circuits.js';
import { createDebugLogger, Fr } from '@aztec/foundation';
import { KernelProver } from '@aztec/kernel-prover';
import { Tx, TxHash } from '@aztec/tx';
import { generateFunctionSelector } from '../abi_coder/index.js';
import { AztecRPCClient, DeployedContract } from '../aztec_rpc_client/index.js';
import { generateContractAddress, selectorToNumber, Signature } from '../circuits.js';
import { Database } from '../database/database.js';
import { TxDao } from '../database/tx_dao.js';
import { KeyStore } from '../key_store/index.js';
import { ContractAbi, FunctionType } from '../noir.js';
import { Synchroniser } from '../synchroniser/index.js';

export class AztecRPCServer implements AztecRPCClient {
  private synchroniser: Synchroniser;
  constructor(
    private keyStore: KeyStore,
    private acirSimulator: AcirSimulator,
    private kernelProver: KernelProver,
    private node: AztecNode,
    private db: Database,
    private log = createDebugLogger('aztec:rpc_server'),
  ) {
    this.synchroniser = new Synchroniser(node, db);
    this.synchroniser.start();
  }

  public async stop() {
    await this.synchroniser.stop();
  }

  public async addAccount() {
    const accountPublicKey = await this.keyStore.addAccount();
    this.log(`adding account ${accountPublicKey.toString()}`);
    await this.synchroniser.addAccount(accountPublicKey);
    return accountPublicKey;
  }

  public async addContracts(contracts: DeployedContract[]) {
    await Promise.all(contracts.map(c => this.db.addContract(c.address, c.portalAddress, c.abi)));
  }

  public getAccounts() {
    return Promise.resolve(this.synchroniser.getAccounts().map(a => a.publicKey));
  }

  public getStorageAt(contract: AztecAddress, storageSlot: Fr) {
    return Promise.resolve([[0]]);
  }

  public getCode(contract: AztecAddress, functionSelector?: Buffer) {
    return this.db.getCode(contract, functionSelector || generateFunctionSelector('constructor', []));
  }

  public async createDeploymentTxRequest(
    abi: ContractAbi,
    args: Fr[],
    portalContract: EthAddress,
    contractAddressSalt: Fr,
    from: AztecAddress,
  ) {
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    const functionData = new FunctionData(
      selectorToNumber(generateFunctionSelector(constructorAbi.name, constructorAbi.parameters)),
      true,
      true,
    );

    const constructorVkHash = Fr.ZERO;
    const functionTreeRoot = Fr.ZERO;
    const contractDeploymentData = new ContractDeploymentData(
      constructorVkHash,
      functionTreeRoot,
      contractAddressSalt,
      portalContract,
    );
    const txContext = new TxContext(false, false, true, contractDeploymentData);
    const fromAddress = from.toBuffer().equals(Fr.ZERO.toBuffer()) ? (await this.keyStore.getAccounts())[0] : from;

    const contractAddress = generateContractAddress(fromAddress, contractAddressSalt, args);
    await this.db.addContract(contractAddress, portalContract, abi, false);

    const txRequestArgs = args.concat(
      Array(ARGS_LENGTH - args.length)
        .fill(0)
        .map(() => new Fr(0n)),
    );

    return new TxRequest(
      fromAddress,
      contractAddress,
      functionData,
      txRequestArgs,
      Fr.random(), // nonce
      txContext,
      Fr.ZERO, // chainId
    );
  }

  public async createTxRequest(functionName: string, args: Fr[], to: AztecAddress, from: AztecAddress) {
    const contract = await this.db.getContract(to);
    if (!contract) {
      throw new Error('Unknown contract.');
    }

    const functionDao = contract.functions.find(f => f.name === functionName);
    if (!functionDao) {
      throw new Error('Unknown function.');
    }

    const functionData = new FunctionData(
      functionDao.selector.readUint32BE(),
      functionDao.functionType === FunctionType.SECRET,
      false,
    );

    const txContext = new TxContext(
      false,
      false,
      true,
      new ContractDeploymentData(Fr.ZERO, Fr.ZERO, Fr.ZERO, new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES))),
    );

    return new TxRequest(
      from,
      to,
      functionData,
      args,
      Fr.random(), // nonce
      txContext,
      Fr.ZERO, // chainId
    );
  }

  public signTxRequest(txRequest: TxRequest) {
    return this.keyStore.signTxRequest(txRequest);
  }

  public async createTx(txRequest: TxRequest, signature: Signature) {
    let contractAddress;

    if (txRequest.to.toBuffer().equals(Fr.ZERO.toBuffer())) {
      contractAddress = generateContractAddress(
        txRequest.from,
        txRequest.txContext.contractDeploymentData.contractAddressSalt,
        txRequest.args,
      );
    } else {
      contractAddress = txRequest.to;
    }

    const contract = await this.db.getContract(contractAddress);

    if (!contract) {
      throw new Error('Unknown contract.');
    }
    const selector = Buffer.alloc(4);
    selector.writeUint32BE(txRequest.functionData.functionSelector);

    const functionDao = contract.functions.find(f => f.selector.equals(selector));
    if (!functionDao) {
      throw new Error('Unknown function.');
    }

    const oldRoots = new OldTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO); // TODO - get old roots from the database/node
    const executionResult = await this.acirSimulator.run(
      txRequest,
      Buffer.from(functionDao.bytecode),
      contract.portalAddress,
      oldRoots,
    );
    const { publicInputs } = await this.kernelProver.prove(
      txRequest as any, // TODO - remove `as any`
      signature,
      executionResult,
      oldRoots as any, // TODO - remove `as any`
    );
    const tx = new Tx(publicInputs, new UInt8Vector(Buffer.alloc(0)), Buffer.alloc(0));
    const dao: TxDao = new TxDao(
      new TxHash(tx.txId),
      undefined,
      undefined,
      txRequest.from,
      undefined,
      txRequest.to,
      '',
    );
    await this.db.addOrUpdateTx(dao);
    return tx;
  }

  public async sendTx(tx: Tx) {
    await this.node.sendTx(tx);
    return new TxHash(tx.txId);
  }

  public getTxReceipt(txHash: TxHash) {
    return this.synchroniser.getTxReceipt(txHash);
  }
}
