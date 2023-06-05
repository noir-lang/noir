RUST_LOG="debug"

if [ -d ${PKG_PATH} ]; then
    rm -rf ${PKG_PATH}
fi

cargo build --lib --release --target x86_64-unknown-linux-gnu
# wasm-bindgen ./target/x86_64-unknown-linux-gnu/release/noir.wasm --out-dir ./pkg/nodejs --typescript --target nodejs
# wasm-bindgen ./target/x86_64-unknown-linux-gnu/release/noir.wasm --out-dir ./pkg/web --typescript --target web
# wasm-opt ./pkg/nodejs/noir_wasm.wasm -o ./pkg/nodejs/noir_wasm.wasm -O
# wasm-opt ./pkg/web/noir_wasm.wasm -o ./pkg/web/noir_wasm.wasm -O

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