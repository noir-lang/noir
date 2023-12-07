import path, { resolve } from 'path';
import webpack from 'webpack';
// in case you run into any typescript error when configuring `devServer`
import 'webpack-dev-server';
import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin';

import HtmlWebpackPlugin from 'html-webpack-plugin';

const config: webpack.Configuration = {
  entry: './index.ts',
  output: {
    path: path.resolve(__dirname, 'dist'),
    globalObject: 'this',
  },
  plugins: [
    new HtmlWebpackPlugin({
      title: 'Noir Wasm',
    }),
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
  devServer: {
    port: 9000,
  },
};

const webConfig: webpack.Configuration = {
  name: 'web',
  ...config,
  output: {
    ...config.output,
    path: path.resolve(__dirname, 'dist/web'),
    library: {
      type: 'window',
    },
  },
};

const nodeConfig: webpack.Configuration = {
  name: 'node',
  ...config,
  output: {
    ...config.output,
    path: path.resolve(__dirname, 'dist/node'),
    library: {
      type: 'commonjs2',
    },
  },
  target: 'node',
};

console.log(nodeConfig);

export default [webConfig, nodeConfig];
