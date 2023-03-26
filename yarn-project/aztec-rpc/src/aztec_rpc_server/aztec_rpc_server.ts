import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { KernelProver } from '@aztec/kernel-prover';
import { Tx } from '@aztec/p2p';
import { generateFunctionSelector } from '../abi_coder/index.js';
import { AztecRPCClient } from '../aztec_rpc_client/index.js';
import {
  AztecAddress,
  ContractDeploymentData,
  EthAddress,
  Fr,
  generateContractAddress,
  OldTreeRoots,
  Signature,
  TxContext,
  TxRequest,
} from '../circuits.js';
import { ContractDao, ContractDataSource } from '../contract_data_source/index.js';
import { KeyStore } from '../key_store/index.js';
import { ContractAbi } from '../noir.js';
import { Synchroniser } from '../synchroniser/index.js';
import { TxHash } from '../tx/index.js';

export class AztecRPCServer implements AztecRPCClient {
  constructor(
    private keyStore: KeyStore,
    private synchroniser: Synchroniser,
    private acirSimulator: AcirSimulator,
    private kernelProver: KernelProver,
    private node: AztecNode,
    private db: ContractDataSource,
  ) {}

  public async addAccount() {
    const accountPublicKey = await this.keyStore.addAccount();
    await this.synchroniser.addAccount(accountPublicKey);
    return accountPublicKey;
  }

  public getAccounts() {
    return Promise.resolve(this.synchroniser.getAccounts().map(a => a.publicKey));
  }

  public getStorageAt(contract: AztecAddress, storageSlot: Fr) {
    return Promise.resolve();
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

    const functionData = {
      functionSelector: generateFunctionSelector(constructorAbi.name, constructorAbi.parameters),
      isSecret: true,
      isContructor: true,
    };

    const constructorVkHash = Fr.ZERO;
    const functionTreeRoot = Fr.ZERO;
    const contractDeploymentData = new ContractDeploymentData(
      constructorVkHash,
      functionTreeRoot,
      contractAddressSalt,
      portalContract,
    );
    const txContext = new TxContext(false, false, false, contractDeploymentData);

    const contractAddress = generateContractAddress(from, contractAddressSalt, args);
    await this.db.addContract(contractAddress, portalContract, abi, false);

    return new TxRequest(
      from,
      AztecAddress.ZERO, // to
      functionData,
      args,
      txContext,
      Fr.random(), // nonce
      Fr.ZERO, // chainId
    );
  }

  public async createTxRequest(functionSelector: Buffer, args: Fr[], to: AztecAddress, from: AztecAddress) {
    const contract = await this.db.getContract(to);
    if (!contract) {
      throw new Error('Unknown contract.');
    }

    const functionDao = this.findFunction(contract, functionSelector);

    const functionData = {
      functionSelector,
      isSecret: functionDao.isSecret,
      isContructor: false,
    };

    const txContext = new TxContext(false, false, false, ContractDeploymentData.EMPTY);

    return new TxRequest(
      from,
      to,
      functionData,
      args,
      txContext,
      Fr.random(), // nonce
      Fr.ZERO, // chainId
    );
  }

  public signTxRequest(txRequest: TxRequest) {
    return this.keyStore.signTxRequest(txRequest);
  }

  public async createTx(txRequest: TxRequest, signature: Signature) {
    let contractAddress;

    if (txRequest.to.equals(AztecAddress.ZERO)) {
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

    const functionDao = this.findFunction(contract, txRequest.functionData.functionSelector);

    const oldRoots = new OldTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO); // TODO - get old roots from the database/node
    const executionResult = await this.acirSimulator.run(
      txRequest,
      Buffer.from(functionDao.bytecode, 'base64'),
      contract.portalAddress,
      oldRoots,
    );
    const { publicInputs } = await this.kernelProver.prove(
      txRequest as any, // TODO - remove `as any`
      signature,
      executionResult,
      oldRoots as any, // TODO - remove `as any`
    );
    // TODO I think the TX should include all the data from the publicInputs + proof
    return new Tx(publicInputs);
  }

  public async sendTx(tx: Tx) {
    await this.node.sendTx(tx);
    return new TxHash(tx.txId);
  }

  public getTxReceipt(txHash: TxHash) {
    return this.synchroniser.getTxReceipt(txHash);
  }

  private findFunction(contract: ContractDao, functionSelector: Buffer) {
    const functionDao = contract.functions.find(f => f.selector.equals(functionSelector));
    if (!functionDao) {
      throw new Error('Unknown function.');
    }
    return functionDao;
  }
}
