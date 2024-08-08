import { type DetectorSync, type IResource, Resource } from '@opentelemetry/resources';

import { NETWORK_NAME } from './attributes.js';
import { getConfigEnvVars } from './config.js';

/**
 * Detector for custom Aztec attributes
 */
class AztecDetector implements DetectorSync {
  detect(): IResource {
    const config = getConfigEnvVars();

    return new Resource({
      [NETWORK_NAME]: config.networkName,
    });
  }
}

export const aztecDetector = new AztecDetector();
