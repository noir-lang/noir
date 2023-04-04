// THIS IS GENERATED CODE, DO NOT EDIT!
/* eslint-disable */
import { EthAddress } from '@aztec/foundation';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { Contract, ContractTxReceipt, EventLog, Options, TxCall, TxSend } from '@aztec/ethereum.js/contract';
import * as Bytes from '@aztec/ethereum.js/contract/bytes.js';
import abi from './UnverifiedDataEmitterAbi.js';
export type ContractDeploymentEvent = {
  aztecAddress: Bytes.Bytes32;
  portalAddress: EthAddress;
  acir: Bytes.Bytes;
};
export type UnverifiedDataEvent = {
  l2BlockNum: bigint;
  sender: EthAddress;
  data: Bytes.Bytes;
};
export interface ContractDeploymentEventLog extends EventLog<ContractDeploymentEvent, 'ContractDeployment'> {}
export interface UnverifiedDataEventLog extends EventLog<UnverifiedDataEvent, 'UnverifiedData'> {}
interface UnverifiedDataEmitterEvents {
  ContractDeployment: ContractDeploymentEvent;
  UnverifiedData: UnverifiedDataEvent;
}
interface UnverifiedDataEmitterEventLogs {
  ContractDeployment: ContractDeploymentEventLog;
  UnverifiedData: UnverifiedDataEventLog;
}
interface UnverifiedDataEmitterTxEventLogs {
  ContractDeployment: ContractDeploymentEventLog[];
  UnverifiedData: UnverifiedDataEventLog[];
}
export interface UnverifiedDataEmitterTransactionReceipt extends ContractTxReceipt<UnverifiedDataEmitterTxEventLogs> {}
interface UnverifiedDataEmitterMethods {
  emitContractDeployment(
    _aztecAddress: Bytes.Bytes32,
    _portalAddress: EthAddress,
    _acir: Bytes.Bytes,
  ): TxSend<UnverifiedDataEmitterTransactionReceipt>;
  emitUnverifiedData(_l2BlockNum: bigint, _data: Bytes.Bytes): TxSend<UnverifiedDataEmitterTransactionReceipt>;
}
export interface UnverifiedDataEmitterDefinition {
  methods: UnverifiedDataEmitterMethods;
  events: UnverifiedDataEmitterEvents;
  eventLogs: UnverifiedDataEmitterEventLogs;
}
export class UnverifiedDataEmitter extends Contract<UnverifiedDataEmitterDefinition> {
  constructor(eth: EthereumRpc, address?: EthAddress, options?: Options) {
    super(eth, abi, address, options);
  }
  deploy(): TxSend<UnverifiedDataEmitterTransactionReceipt> {
    return super.deployBytecode(
      '0x608060405234801561001057600080fd5b50610258806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c8063ec1c72ff1461003b578063f3f53b3a14610050575b600080fd5b61004e61004936600461013f565b610063565b005b61004e61005e36600461018b565b6100ac565b336001600160a01b0316837fb72b85ef2a843244f5ea1955248b0ac363732d2e6a98cc3641084dd5718ad8b5848460405161009f9291906101f3565b60405180910390a3505050565b826001600160a01b0316847f30735f52bf1ff4d542583f894902a2775489f644723a4b3cca1de6d6eabd0c4a84846040516100e89291906101f3565b60405180910390a350505050565b60008083601f84011261010857600080fd5b50813567ffffffffffffffff81111561012057600080fd5b60208301915083602082850101111561013857600080fd5b9250929050565b60008060006040848603121561015457600080fd5b83359250602084013567ffffffffffffffff81111561017257600080fd5b61017e868287016100f6565b9497909650939450505050565b600080600080606085870312156101a157600080fd5b8435935060208501356001600160a01b03811681146101bf57600080fd5b9250604085013567ffffffffffffffff8111156101db57600080fd5b6101e7878288016100f6565b95989497509550505050565b60208152816020820152818360408301376000818301604090810191909152601f909201601f1916010191905056fea2646970667358221220b4668eba21fe4e02e4a0ab3b4d1c9cdd827470b2961b1e2da0dba0ace9f4331064736f6c63430008120033',
    ) as any;
  }
}
export var UnverifiedDataEmitterAbi = abi;
