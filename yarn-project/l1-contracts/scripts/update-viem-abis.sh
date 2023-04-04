set -euo pipefail;

echo -ne "/**\n * Rollup ABI for viem.\n */\nexport const RollupAbi = " > ./src/viem-abis/RollupAbi.ts;
jq -j '.abi' ../../l1-contracts/out/Rollup.sol/Rollup.json >> ./src/viem-abis/RollupAbi.ts;
echo " as const;" >> ./src/viem-abis/RollupAbi.ts;

echo -ne "/**\n * UnverifiedDataEmitter ABI for viem.\n */\nexport const UnverifiedDataEmitterAbi = " > ./src/viem-abis/UnverifiedDataEmitterAbi.ts;
jq -j '.abi' ../../l1-contracts/out/UnverifiedDataEmitter.sol/UnverifiedDataEmitter.json >> ./src/viem-abis/UnverifiedDataEmitterAbi.ts;
echo " as const;" >> ./src/viem-abis/UnverifiedDataEmitterAbi.ts;
