import { PrivateTokenContractAbi } from './artifacts/private_token.js';
import { PXE, createPXEClient } from '@aztec/aztec.js';
import { ContractAbi } from '@aztec/foundation/abi';

// update this if using a different contract

export const contractAbi: ContractAbi = PrivateTokenContractAbi;

export const PXE_URL: string = process.env.PXE_URL || 'http://localhost:8080';
export const pxe: PXE = createPXEClient(PXE_URL);

export const CONTRACT_ADDRESS_PARAM_NAMES = ['owner', 'contract_address', 'recipient'];
export const FILTERED_FUNCTION_NAMES = ['compute_note_hash_and_nullifier'];
