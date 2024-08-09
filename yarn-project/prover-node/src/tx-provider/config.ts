import { type ConfigMappingsType, getConfigFromMappings } from '@aztec/foundation/config';

export type TxProviderConfig = {
  txProviderNodeUrl: string | undefined;
};

export const txProviderConfigMappings: ConfigMappingsType<TxProviderConfig> = {
  txProviderNodeUrl: {
    env: 'TX_PROVIDER_NODE_URL',
    description: 'The URL of the tx provider node',
    parseEnv: (val: string) => val,
  },
};

export function getTxProviderConfigFromEnv(): TxProviderConfig {
  return getConfigFromMappings<TxProviderConfig>(txProviderConfigMappings);
}
