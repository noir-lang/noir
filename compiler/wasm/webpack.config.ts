import { resolve, join } from 'path';
import webpack from 'webpack';
import type { Configuration as DevServerConfiguration } from 'webpack-dev-server';
import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin';
import HtmlWebpackPlugin from 'html-webpack-plugin';
import CopyWebpackPlugin from 'copy-webpack-plugin';

const config: webpack.Configuration = {
  output: {
    path: resolve(__dirname, 'dist'),
  },
  mode: 'development',
  devtool: 'source-map',
  optimization: {
    minimize: false,
  },
  resolve: {
    extensions: ['.cts', '.mts', '.ts', '.js', '.json', '.wasm'],
    fallback: {
      path: require.resolve('path-browserify'),
      stream: require.resolve('readable-stream'),
      fs: require.resolve('browserify-fs'),
      buffer: require.resolve('buffer'),
    },
  },
};

const devServerConfig: DevServerConfiguration = {
  static: join(__dirname, 'dist'),
};

const webConfig: webpack.Configuration = {
  name: 'web',
  entry: './src/index.mts',
  ...config,
  experiments: { asyncWebAssembly: true, outputModule: true },
  output: {
    filename: 'main.mjs',
    ...config.output,
    path: resolve(__dirname, 'dist/web'),
    library: {
      type: 'module',
    },
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: resolve(__dirname),
      outDir: resolve(__dirname, './build/esm'),
      extraArgs: '--target web',
      forceMode: process.env.WASM_OPT === 'true' ? 'production' : 'development',
    }),
    new HtmlWebpackPlugin({
      title: 'Noir Wasm ESM',
    }),
    new webpack.DefinePlugin({
      'process.env.NODE_DEBUG': JSON.stringify(process.env.NODE_DEBUG),
    }),
    new webpack.ProvidePlugin({
      process: 'process/browser',
    }),
    new webpack.ProvidePlugin({
      Buffer: ['buffer', 'Buffer'],
    }),
  ],
  module: {
    rules: [
      {
        test: /.m?ts$/,
        loader: 'ts-loader',
        options: {
          configFile: 'tsconfig.esm.json',
        },
        exclude: /node_modules/,
      },
      {
        test: /\.wasm$/,
        type: 'asset/inline',
      },
    ],
  },
  devServer: devServerConfig,
  resolve: {
    ...config.resolve,
    alias: {
      fs: 'memfs',
    },
  },
};

const nodeConfig: webpack.Configuration = {
  name: 'node',
  entry: './src/index.cts',
  ...config,
  output: {
    ...config.output,
    path: resolve(__dirname, 'dist/node'),
    library: {
      type: 'commonjs2',
    },
  },
  target: 'node',
  plugins: [
    new WasmPackPlugin({
      crateDirectory: resolve(__dirname),
      outDir: resolve(__dirname, './build/cjs'),
      extraArgs: '--target nodejs',
      forceMode: process.env.WASM_OPT === 'true' ? 'production' : 'development',
    }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: resolve(__dirname, './build/cjs/index_bg.wasm'),
          to: resolve(__dirname, 'dist/node/index_bg.wasm'),
        },
      ],
    }),
  ],
  module: {
    rules: [
      {
        test: /.c?ts$/,
        loader: 'ts-loader',
        options: {
          configFile: 'tsconfig.webpack.json',
        },
        exclude: /node_modules/,
      },
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
    ],
  },
};

export default [webConfig, nodeConfig];
