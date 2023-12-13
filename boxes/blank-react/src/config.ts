import { ContractArtifact, PXE, createPXEClient } from '@aztec/aztec.js';
import { BlankContractArtifact } from './artifacts/Blank.js';

// update this if using a different contract

export const contractArtifact: ContractArtifact = BlankContractArtifact;

export const PXE_URL: string = process.env.PXE_URL || 'http://localhost:8080';
export const pxe: PXE = createPXEClient(PXE_URL);

export const CONTRACT_ADDRESS_PARAM_NAMES = ['address'];
