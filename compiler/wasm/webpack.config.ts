import path from 'path';
import webpack from 'webpack';
// in case you run into any typescript error when configuring `devServer`
import 'webpack-dev-server';
import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin';

import HtmlWebpackPlugin from 'html-webpack-plugin';

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
  devServer: {
    port: 9000,
  },
  resolve: {
    extensions: ['.cts', '.mts', '.ts', '.js', '.json', '.wasm'],
    fallback: {
      assert: require.resolve('assert'),
      buffer: require.resolve('buffer'),
      path: require.resolve('path-browserify'),
      process: require.resolve('process/browser'),
      stream: require.resolve('readable-stream'),
      url: require.resolve('url'),
      util: require.resolve('util'),
    },
  },
};

const webConfig: webpack.Configuration = {
  name: 'web',
  entry: './noir_wasm/src/index.mts',
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
      crateDirectory: path.resolve(__dirname, './noir_wasm'),
      outDir: path.resolve(__dirname, './noir_wasm/esm'),
    }),
    new HtmlWebpackPlugin({
      title: 'Noir Wasm',
    }),
    new webpack.DefinePlugin({
      'process.env.NODE_DEBUG': JSON.stringify(process.env.NODE_DEBUG),
    }),
  ],

  externals: {
    fs: 'window.fs',
  },
  module: {
    rules: [
      // {
      //   test: /\.(shim\.)?[cmj]tsx?$/,
      //   exclude: /node_modules/,
      //   use: {
      //     loader: 'ts-loader',
      //     options: {
      //       configFile: 'noir_wasm/tsconfig.esm.json', // or tsconfig.esm.json
      //     },
      //   },
      // },
      {
        test: /\.(shim\.)?[cmjt]t?s?$/,
        exclude: /node_modules/,
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
};

const nodeConfig: webpack.Configuration = {
  name: 'node',
  entry: './noir_wasm/src/index.cts',
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
      crateDirectory: path.resolve(__dirname, './noir_wasm'),
      outDir: path.resolve(__dirname, './noir_wasm/cjs'),
    }),
  ],
  module: {
    rules: [
      {
        test: /\.(shim\.)?[cmjt]t?s?$/,
        exclude: /node_modules/,
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
};

export default [webConfig, nodeConfig];
