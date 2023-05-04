set -euo pipefail;

target_dir=./generated

# create target dir if it doesn't exist
mkdir -p "$target_dir";

echo -ne "/**\n * DecoderHelper ABI.\n */\nexport const DecoderHelperAbi = " > "$target_dir/DecoderHelperAbi.ts";
jq -j '.abi' ../../l1-contracts/out/DecoderHelper.sol/DecoderHelper.json >> "$target_dir/DecoderHelperAbi.ts";
echo " as const;" >> "$target_dir/DecoderHelperAbi.ts";
echo -ne "/**\n * DecoderHelper bytecode.\n */\nexport const DecoderHelperBytecode = \"" > "$target_dir/DecoderHelperBytecode.ts";
jq -j '.bytecode.object' ../../l1-contracts/out/DecoderHelper.sol/DecoderHelper.json >> "$target_dir/DecoderHelperBytecode.ts";
echo "\";" >> "$target_dir/DecoderHelperBytecode.ts";

echo -ne "/**\n * Rollup ABI.\n */\nexport const RollupAbi = " > "$target_dir/RollupAbi.ts";
jq -j '.abi' ../../l1-contracts/out/Rollup.sol/Rollup.json >> "$target_dir/RollupAbi.ts";
echo " as const;" >> "$target_dir/RollupAbi.ts";

echo -ne "/**\n * Rollup bytecode.\n */\nexport const RollupBytecode = '" > "$target_dir/RollupBytecode.ts";
jq -j '.bytecode.object' ../../l1-contracts/out/Rollup.sol/Rollup.json >> "$target_dir/RollupBytecode.ts";
echo "' as const;" >> "$target_dir/RollupBytecode.ts";

echo -ne "/**\n * UnverifiedDataEmitter ABI.\n */\nexport const UnverifiedDataEmitterAbi = " > "$target_dir/UnverifiedDataEmitterAbi.ts";
jq -j '.abi' ../../l1-contracts/out/UnverifiedDataEmitter.sol/UnverifiedDataEmitter.json >> "$target_dir/UnverifiedDataEmitterAbi.ts";
echo " as const;" >> "$target_dir/UnverifiedDataEmitterAbi.ts";

echo -ne "/**\n * UnverifiedDataEmitter bytecode.\n */\nexport const UnverifiedDataEmitterBytecode = '" > "$target_dir/UnverifiedDataEmitterBytecode.ts";
jq -j '.bytecode.object' ../../l1-contracts/out/UnverifiedDataEmitter.sol/UnverifiedDataEmitter.json >> "$target_dir/UnverifiedDataEmitterBytecode.ts";
echo "' as const;" >> "$target_dir/UnverifiedDataEmitterBytecode.ts";

echo -ne "export * from './DecoderHelperAbi.js';\nexport * from './DecoderHelperBytecode.js';\n" > "$target_dir/index.ts";
echo -ne "export * from './RollupAbi.js';\nexport * from './RollupBytecode.js';\n" >> "$target_dir/index.ts";
echo -ne "export * from './UnverifiedDataEmitterAbi.js';\nexport * from './UnverifiedDataEmitterBytecode.js';" >> "$target_dir/index.ts";

echo "Successfully generated TS artifacts!";
