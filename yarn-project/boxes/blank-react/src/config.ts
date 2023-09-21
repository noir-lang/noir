import { BlankContractAbi } from './artifacts/blank.js';
import { AztecRPC, createAztecRpcClient } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';

// update this if using a different contract

export const contractAbi: ContractAbi = BlankContractAbi;

export const SANDBOX_URL: string = process.env.SANDBOX_URL || 'http://localhost:8080';
export const rpcClient: AztecRPC = createAztecRpcClient(SANDBOX_URL);

export const CONTRACT_ADDRESS_PARAM_NAMES = ['address'];
export const FILTERED_FUNCTION_NAMES = [];
