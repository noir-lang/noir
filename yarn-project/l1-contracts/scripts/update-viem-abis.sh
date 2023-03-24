set -euo pipefail;

echo -ne "/**\n * Rollup ABI for viem.\n */\nexport const RollupAbi = " > ./src/viem-abis/RollupAbi.ts;
jq -j '.abi' ../../l1-contracts/out/Rollup.sol/Rollup.json >> ./src/viem-abis/RollupAbi.ts;
echo " as const;" >> ./src/viem-abis/RollupAbi.ts;

echo -ne "/**\n * Yeeter ABI for viem.\n */\nexport const YeeterAbi = " > ./src/viem-abis/YeeterAbi.ts;
jq -j '.abi' ../../l1-contracts/out/Yeeter.sol/Yeeter.json >> ./src/viem-abis/YeeterAbi.ts;
echo " as const;" >> ./src/viem-abis/YeeterAbi.ts;
