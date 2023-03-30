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
import { hashVK } from '@aztec/circuits.js/abis';
import { CircuitsWasm } from '@aztec/circuits.js/wasm';
import { createDebugLogger, Fr } from '@aztec/foundation';
import { KernelProver } from '@aztec/kernel-prover';
import { Tx, TxHash } from '@aztec/tx';
import { generateFunctionSelector } from '../abi_coder/index.js';
import { AztecRPCClient, DeployedContract } from '../aztec_rpc_client/index.js';
import { selectorToNumber, Signature } from '../circuits.js';
import { ContractTree } from '../contract_tree/index.js';
import { Database } from '../database/database.js';
import { TxDao } from '../database/tx_dao.js';
import { KeyStore } from '../key_store/index.js';
import { ContractAbi, FunctionType } from '../noir.js';
import { Synchroniser } from '../synchroniser/index.js';
import { TxReceipt, TxStatus } from '../tx/index.js';

/**
 * Implements a remote Aztec RPC client provider.
 * Combines our major components into one API.
 */
export class AztecRPCServer implements AztecRPCClient {
  private synchroniser: Synchroniser;

  constructor(
    private keyStore: KeyStore,
    private acirSimulator: AcirSimulator,
    private kernelProver: KernelProver,
    private node: AztecNode,
    private db: Database,
    private circuitsWasm: CircuitsWasm,
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
    const trees = contracts.map(c => ContractTree.fromAddress(c.address, c.abi, c.portalAddress, this.circuitsWasm));
    await Promise.all(trees.map(t => this.db.addContract(t.contract)));
  }

  public getAccounts() {
    return Promise.resolve(this.synchroniser.getAccounts().map(a => a.publicKey));
  }

  public async getStorageAt(contract: AztecAddress, storageSlot: Fr) {
    const notes = await this.db.getNotes(contract, storageSlot);
    return notes.map(n => n.notePreimage.items.map(item => item.value));
  }

  /**
   * Is an L2 contract deployed at this address?
   * @param contractAddress - The contract data address.
   * @returns Whether the contract was deployed.
   */
  public async isContractDeployed(contractAddress: AztecAddress): Promise<boolean> {
    return !!(await this.node.getContractData(contractAddress));
  }

  public async createDeploymentTxRequest(
    abi: ContractAbi,
    args: any[],
    portalContract: EthAddress,
    contractAddressSalt: Fr,
    from: AztecAddress,
  ) {
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    if (!constructorAbi.verificationKey) {
      throw new Error('Missing verification key for the constructor.');
    }

    const txRequestArgs = args.concat(
      Array(ARGS_LENGTH - args.length)
        .fill(0)
        .map(() => new Fr(0n)),
    );

    const fromAddress = from.equals(AztecAddress.ZERO) ? (await this.keyStore.getAccounts())[0] : from;
    const contractTree = ContractTree.new(
      abi,
      txRequestArgs,
      portalContract,
      contractAddressSalt,
      fromAddress,
      this.circuitsWasm,
    );
    const contract = contractTree.contract;

    const functionData = new FunctionData(
      selectorToNumber(generateFunctionSelector(constructorAbi.name, constructorAbi.parameters)),
      true,
      true,
    );

    const constructorVkHash = Fr.fromBuffer(
      hashVK(this.circuitsWasm, Buffer.from(constructorAbi.verificationKey, 'hex')),
    );

    const contractDeploymentData = new ContractDeploymentData(
      constructorVkHash,
      contractTree.getFunctionTreeRoot(),
      contractAddressSalt,
      portalContract,
    );

    const txContext = new TxContext(false, false, true, contractDeploymentData);

    await this.db.addContract(contract);

    return new TxRequest(
      fromAddress,
      contract.address,
      functionData,
      txRequestArgs,
      Fr.random(), // nonce
      txContext,
      Fr.ZERO, // chainId
    );
  }

  public async createTxRequest(functionName: string, args: any[], to: AztecAddress, from: AztecAddress) {
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
    const contractAddress = txRequest.to;
    const contract = await this.db.getContract(txRequest.to);

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
      contractAddress,
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
    const dao: TxDao = new TxDao(tx.txHash, undefined, undefined, txRequest.from, undefined, txRequest.to, '');
    await this.db.addOrUpdateTx(dao);
    return tx;
  }

  /**
   * Send a transaction.
   * @param tx - The transaction
   * @returns A hash of the transaction, used to identify it.
   */
  public async sendTx(tx: Tx): Promise<TxHash> {
    await this.node.sendTx(tx);
    return tx.txHash;
  }
  /**
   * Fetchs a transaction receipt for a tx
   * @param txHash - The transaction hash
   * @returns A recipt of the transaction
   */
  public async getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    const localTx = await this.synchroniser.getTxByHash(txHash);
    const partialReceipt = {
      txHash: txHash,
      blockHash: localTx?.blockHash,
      blockNumber: localTx?.blockNumber,
      from: localTx?.from,
      to: localTx?.to,
      contractAddress: localTx?.contractAddress,
      error: '',
    };

    if (localTx && localTx.blockHash) {
      return {
        ...partialReceipt,
        status: TxStatus.MINED,
      };
    }

    const pendingTx = await this.node.getPendingTxByHash(txHash);
    if (pendingTx) {
      return {
        ...partialReceipt,
        status: TxStatus.PENDING,
      };
    }

    // if the transaction mined it will be removed from the pending pool and there is a race condition here as the synchroniser will not have the tx as mined yet, so it will appear dropped
    // until the synchroniser picks this up

    const remoteBlockHeight = await this.node.getBlockHeight();
    const accountBlockHeight = this.synchroniser.getAccount(localTx.from)?.syncedTo || 0;

    if (localTx && remoteBlockHeight > accountBlockHeight) {
      // there is a pending L2 block, which means the transaction will not be in the tx pool but may be awaiting mine on L1
      return {
        ...partialReceipt,
        status: TxStatus.PENDING,
      };
    }

    // TODO we should refactor this once the node can store transactions. At that point we should query the node and not deal with block heights.

    return {
      ...partialReceipt,
      status: TxStatus.DROPPED,
    };
  }
}
