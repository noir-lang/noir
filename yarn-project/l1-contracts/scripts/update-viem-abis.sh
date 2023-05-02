set -euo pipefail;

echo -ne "/**\n * DecoderHelper ABI for viem.\n */\nexport const DecoderHelperAbi = " > ./src/viem-contracts/DecoderHelper.ts;
jq -j '.abi' ../../l1-contracts/out/DecoderHelper.sol/DecoderHelper.json >> ./src/viem-contracts/DecoderHelper.ts;
echo " as const;" >> ./src/viem-contracts/DecoderHelper.ts;
echo -ne "/**\n * DecoderHelper Bytecode for viem.\n */\nexport const DecoderHelperBytecode = \"" >> ./src/viem-contracts/DecoderHelper.ts;
jq -j '.bytecode.object' ../../l1-contracts/out/DecoderHelper.sol/DecoderHelper.json >> ./src/viem-contracts/DecoderHelper.ts;
echo "\";" >> ./src/viem-contracts/DecoderHelper.ts;

echo -ne "/**\n * Rollup ABI for viem.\n */\nexport const RollupAbi = " > ./src/viem-contracts/Rollup.ts;
jq -j '.abi' ../../l1-contracts/out/Rollup.sol/Rollup.json >> ./src/viem-contracts/Rollup.ts;
echo " as const;" >> ./src/viem-contracts/Rollup.ts;
echo -ne "/**\n * Rollup Bytecode for viem.\n */\nexport const RollupBytecode = \"" >> ./src/viem-contracts/Rollup.ts;
jq -j '.bytecode.object' ../../l1-contracts/out/Rollup.sol/Rollup.json >> ./src/viem-contracts/Rollup.ts;
echo "\";" >> ./src/viem-contracts/Rollup.ts;

echo -ne "/**\n * UnverifiedDataEmitter ABI for viem.\n */\nexport const UnverifiedDataEmitterAbi = " > ./src/viem-contracts/UnverifiedDataEmitter.ts;
jq -j '.abi' ../../l1-contracts/out/UnverifiedDataEmitter.sol/UnverifiedDataEmitter.json >> ./src/viem-contracts/UnverifiedDataEmitter.ts;
echo " as const;" >> ./src/viem-contracts/UnverifiedDataEmitter.ts;
echo -ne "/**\n * UnverifiedDataEmitter Bytecode for viem.\n */\nexport const UnverifiedDataEmitterBytecode = \"" >> ./src/viem-contracts/UnverifiedDataEmitter.ts;
jq -j '.bytecode.object' ../../l1-contracts/out/UnverifiedDataEmitter.sol/UnverifiedDataEmitter.json >> ./src/viem-contracts/UnverifiedDataEmitter.ts;
echo "\";" >> ./src/viem-contracts/UnverifiedDataEmitter.ts;