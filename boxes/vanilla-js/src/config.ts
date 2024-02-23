import { createPXEClient, waitForPXE } from '@aztec/aztec.js';
import { VanillaContractArtifact } from '../artifacts/Vanilla';

class PXE {
  pxeUrl;
  pxe;

  constructor() {
    this.pxeUrl = process.env.PXE_URL || 'http://localhost:8080';
    this.pxe = createPXEClient(this.pxeUrl);
  }

  async setupPxe() {
    await waitForPXE(this.pxe);
    return this.pxe;
  }

  getPxeUrl() {
    return this.pxeUrl;
  }

  getPxe() {
    return this.pxe;
  }
}

export const pxe = new PXE();
export const contractArtifact = VanillaContractArtifact;
export const CONTRACT_ADDRESS_PARAM_NAMES = ['address'];
