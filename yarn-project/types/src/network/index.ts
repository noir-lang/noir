/**
 * Deployed aztec networks & their names.
 */
export enum Network {
  DEVNET = 'devnet',
  PROVERNET = 'provernet',
}

/**
 * Map of basic contracts deployed for each network.
 */
export const L2BasicContractsMap = {
  [Network.DEVNET]: {
    devCoin: 'TokenContract',
    devCoinBridge: 'TokenBridgeContract',
    devCoinFpc: 'FPCContract',
    counter: 'CounterContract',
  },
  [Network.PROVERNET]: {
    devCoin: 'TokenContract',
    devCoinBridge: 'TokenBridgeContract',
    devCoinFpc: 'FPCContract',
    counter: 'CounterContract',
  },
};
