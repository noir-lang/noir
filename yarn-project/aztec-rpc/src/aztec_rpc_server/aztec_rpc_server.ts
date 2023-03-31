import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
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
import { ContractAbi, FunctionType } from '@aztec/noir-contracts';
import { Tx, TxHash } from '@aztec/tx';
import { generateFunctionSelector } from '../abi_coder/index.js';
import { AztecRPCClient, DeployedContract } from '../aztec_rpc_client/index.js';
import { Signature } from '../circuits.js';
import { toContractDao } from '../contract_database/index.js';
import { ContractTree } from '../contract_tree/index.js';
import { Database } from '../database/database.js';
import { TxDao } from '../database/tx_dao.js';
import { KeyStore } from '../key_store/index.js';
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
    bbWasm: BarretenbergWasm,
    private log = createDebugLogger('aztec:rpc_server'),
  ) {
    this.synchroniser = new Synchroniser(node, db, bbWasm);
    this.synchroniser.start();
  }

  public async stop() {
    await this.synchroniser.stop();
  }

  public async addAccount() {
    const accountAddress = await this.keyStore.addAccount();
    const accountPrivateKey = await this.keyStore.getAccountPrivateKey(accountAddress);
    this.log(`adding account ${accountAddress.toString()}`);
    await this.synchroniser.addAccount(accountPrivateKey);
    return accountAddress;
  }

  public async addContracts(contracts: DeployedContract[]) {
    const contractDaos = contracts.map(c => toContractDao(c.abi, c.address, c.portalContract));
    await Promise.all(contractDaos.map(c => this.db.addContract(c)));
  }

  public async getAccounts(): Promise<AztecAddress[]> {
    const accounts = this.synchroniser.getAccounts();
    return await Promise.all(accounts.map(a => a.getPublicKey().toAddress()));
  }

  public async getStorageAt(contract: AztecAddress, storageSlot: Fr) {
    const txAuxData = await this.db.getTxAuxData(contract, storageSlot);
    return txAuxData.map(d => d.notePreimage.items.map(item => item.value));
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
      generateFunctionSelector(constructorAbi.name, constructorAbi.parameters),
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
      functionDao.selector,
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
    const accountState = this.synchroniser.getAccount(txRequest.from);
    if (!accountState) {
      throw new Error('Cannot create tx for an unauthorized account.');
    }

    const contractAddress = txRequest.to;
    const contract = await this.db.getContract(txRequest.to);
    if (!contract) {
      throw new Error('Unknown contract.');
    }
    const selector = txRequest.functionData.functionSelector;

    const functionDao = contract.functions.find(f => f.selector.equals(selector));
    if (!functionDao) {
      throw new Error('Unknown function.');
    }

    const oldRoots = new OldTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO); // TODO - get old roots from the database/node
    const executionResult = await this.acirSimulator.run(
      txRequest,
      functionDao,
      contractAddress,
      contract.portalContract,
      oldRoots,
    );
    const { publicInputs } = await this.kernelProver.prove(
      txRequest as any, // TODO - remove `as any`
      signature,
      executionResult,
      oldRoots as any, // TODO - remove `as any`
    );

    const unverifiedData = accountState.createUnverifiedData(contractAddress, executionResult.preimages.newNotes);
    const tx = new Tx(publicInputs, new UInt8Vector(Buffer.alloc(0)), unverifiedData);

    const [toContract, newContract] = txRequest.functionData.isConstructor
      ? [undefined, contractAddress]
      : [contractAddress, undefined];
    const dao = new TxDao(tx.txHash, undefined, undefined, txRequest.from, toContract, newContract, '');
    await this.db.addTx(dao);

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

    if (localTx?.blockHash) {
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

    const accountState = this.synchroniser.getAccount(localTx.from);
    if (accountState && !(await accountState.isSynchronised())) {
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
      error: 'Tx dropped by P2P node.',
    };
  }
}
