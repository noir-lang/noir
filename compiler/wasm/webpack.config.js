const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
  entry: './index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'index.js',
    publicPath: '',
    globalObject: 'this',
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, './noir_wasm'),
    }),
  ],
  mode: 'development',
  devtool: 'source-map',
  experiments: {
    asyncWebAssembly: true,
  },
  optimization: {
    minimize: false,
  },
};
