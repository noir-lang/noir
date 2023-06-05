RUST_LOG="debug"

if [ -d ${PKG_PATH} ]; then
    rm -rf ${PKG_PATH}
fi

cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen ./target/wasm32-unknown-unknown/release/acvm_simulator.wasm --out-dir ./pkg/nodejs --typescript --target nodejs
wasm-bindgen ./target/wasm32-unknown-unknown/release/acvm_simulator.wasm --out-dir ./pkg/web --typescript --target web
wasm-opt ./pkg/nodejs/acvm_simulator_bg.wasm -o ./pkg/nodejs/acvm_simulator_bg.wasm -O
wasm-opt ./pkg/web/acvm_simulator_bg.wasm -o ./pkg/web/acvm_simulator_bg.wasm -O

# wasm-pack build crates/wasm --mode no-install --scope noir-lang --target web --out-dir pkg/web --release
# echo "Web build complete. Directory contents:"
# ls -la ${PKG_PATH}/web

# wasm-pack build crates/wasm --mode no-install --scope noir-lang --target nodejs --out-dir pkg/nodejs --release
# echo "NodeJS build complete. Directory contents:"
# ls -la ${PKG_PATH}/nodejs

if [ -n ${COMMIT_SHORT} ]; then
    VERSION_APPENDIX="-${COMMIT_SHORT}"
else
    VERSION_APPENDIX="-NOGIT"
fi

# NOTE: This is not working
echo "VERSION_APPENDIX = ${VERSION_APPENDIX}"

jq -s '.[0] * .[1]' ${PKG_PATH}/nodejs/package.json ${PKG_PATH}/web/package.json | jq '.files = ["nodejs", "web", "package.json"]' | jq ".version += \"${VERSION_APPENDIX}\"" | jq '.main = "./nodejs/" + .main | .module = "./web/" + .module | .types = "./web/" + .types | .peerDependencies = { "@noir-lang/noir-source-resolver": "1.1.2" }' | tee ${PKG_PATH}/package.json

rm ${PKG_PATH}/nodejs/package.json ${PKG_PATH}/nodejs/.gitignore
rm ${PKG_PATH}/web/package.json ${PKG_PATH}/web/.gitignore
cat ${PKG_PATH}/package.json