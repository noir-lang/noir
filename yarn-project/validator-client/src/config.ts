import { NULL_KEY } from '@aztec/ethereum';
import { type ConfigMappingsType, booleanConfigHelper, getConfigFromMappings } from '@aztec/foundation/config';

/**
 * The Validator Configuration
 */
export interface ValidatorClientConfig {
  /** The private key of the validator participating in attestation duties */
  validatorPrivateKey: string;

  /** Do not run the validator */
  disableValidator: boolean;
}

export const validatorClientConfigMappings: ConfigMappingsType<ValidatorClientConfig> = {
  validatorPrivateKey: {
    env: 'VALIDATOR_PRIVATE_KEY',
    parseEnv: (val: string) => (val ? `0x${val.replace('0x', '')}` : NULL_KEY),
    description: 'The private key of the validator participating in attestation duties',
  },
  disableValidator: {
    env: 'VALIDATOR_DISABLED',
    description: 'Do not run the validator',
    ...booleanConfigHelper(),
  },
};

/**
 * Returns the prover configuration from the environment variables.
 * Note: If an environment variable is not set, the default value is used.
 * @returns The validator configuration.
 */
export function getProverEnvVars(): ValidatorClientConfig {
  return getConfigFromMappings<ValidatorClientConfig>(validatorClientConfigMappings);
}
