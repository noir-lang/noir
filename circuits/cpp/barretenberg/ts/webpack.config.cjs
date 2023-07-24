/**
 * Builds both the web and node version of the worker, and outputs it to the dest directory.
 * NOTE: Currently only runs on web, has issues with translating node imports to require.
 * Currently node passes only through typescript compiler.
 */
const path = require('path');
const ResolveTypeScriptPlugin = require('resolve-typescript-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');
const webpack = require('webpack');
const { resolve } = path;

const buildTarget = process.env.BUILD_TARGET;
const isNode = buildTarget === 'node';
const configFile = path.resolve(__dirname, `./tsconfig.${buildTarget}.json`);

module.exports = {
  mode: 'production',
  entry: './src/index.ts',
  target: isNode ? 'node' : 'web',
  output: {
    path: resolve(__dirname, `./dest/${buildTarget}`),
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.ts?$/,
        use: [{ loader: 'ts-loader', options: { transpileOnly: true, onlyCompileBundledFiles: true, configFile } }],
      },
    ],
  },
  resolve: {
    plugins: [new ResolveTypeScriptPlugin(), new TsconfigPathsPlugin({ configFile })],
  },
  optimization: {
    minimize: isNode,
  },
  plugins: [
    new webpack.DefinePlugin({ 'process.env.NODE_DEBUG': false }),
    new webpack.ProvidePlugin({ Buffer: ['buffer', 'Buffer'] }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: `../cpp/build-wasm/bin/barretenberg.wasm`,
          to: '../barretenberg.wasm',
        },
        {
          from: `../cpp/build-wasm-threads/bin/barretenberg.wasm`,
          to: '../barretenberg-threads.wasm',
        },
      ],
    }),
  ],
};
