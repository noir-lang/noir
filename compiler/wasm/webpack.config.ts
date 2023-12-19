import path from 'path';
import webpack from 'webpack';
import 'webpack-dev-server';
import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin';

import HtmlWebpackPlugin from 'html-webpack-plugin';
import CopyWebpackPlugin from 'copy-webpack-plugin';

const config: webpack.Configuration = {
  output: {
    path: path.resolve(__dirname, 'dist'),
    globalObject: 'this',
  },
  mode: 'development',
  devtool: 'source-map',
  experiments: {
    asyncWebAssembly: true,
  },
  optimization: {
    minimize: false,
  },
  resolve: {
    extensions: ['.cts', '.mts', '.ts', '.js', '.json', '.wasm'],
    fallback: {
      // assert: require.resolve('assert'),
      // buffer: require.resolve('buffer'),
      path: require.resolve('path-browserify'),
      // process: require.resolve('process/browser'),
      stream: require.resolve('readable-stream'),
      // url: require.resolve('url'),
      // util: require.resolve('util'),
      fs: require.resolve('browserify-fs'),
    },
  },
};

const webConfig: webpack.Configuration = {
  name: 'web',
  entry: './src/index.mts',
  ...config,
  output: {
    ...config.output,
    path: path.resolve(__dirname, 'dist/web'),
    library: {
      type: 'window',
    },
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname),
      outDir: path.resolve(__dirname, './build/esm'),
      forceMode: 'production',
    }),
    new CopyWebpackPlugin({
      patterns: [{ from: path.resolve(__dirname, 'public'), to: path.resolve(__dirname, 'dist/web/public') }],
    }),
    new HtmlWebpackPlugin({
      title: 'Noir Wasm ESM',
    }),
    new webpack.DefinePlugin({
      'process.env.NODE_DEBUG': JSON.stringify(process.env.NODE_DEBUG),
    }),
  ],
  module: {
    rules: [
      {
        test: /\.(shim\.)?[cmjt]t?s?$/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: ['@babel/preset-env', '@babel/preset-typescript'],
            plugins: ['@babel/plugin-proposal-class-properties', '@babel/plugin-transform-private-methods'],
          },
        },
      },
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
    ],
  },
  devServer: {
    static: path.join(__dirname, 'dist'),
  },
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
    path: path.resolve(__dirname, 'dist/node'),
    library: {
      type: 'commonjs2',
    },
  },
  target: 'node',
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname),
      outDir: path.resolve(__dirname, './build/cjs'),
      extraArgs: '--target nodejs',
      forceMode: 'production',
    }),
  ],
  module: {
    rules: [
      {
        test: /\.(shim\.)?[cmjt]t?s?$/,
        use: {
          loader: 'ts-loader',
        },
      },
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
    ],
  },
};

export default [webConfig, nodeConfig];
