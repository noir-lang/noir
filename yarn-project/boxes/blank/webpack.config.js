import CopyWebpackPlugin from 'copy-webpack-plugin';
import { createRequire } from 'module';
import { dirname, resolve } from 'path';
import ResolveTypeScriptPlugin from 'resolve-typescript-plugin';
import { fileURLToPath } from 'url';
import webpack from 'webpack';

const require = createRequire(import.meta.url);

export default (_, argv) => ({
  target: 'web',
  mode: 'production',
  devtool: false,
  entry: {
    main: './src/index.ts',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: [
          {
            loader: 'ts-loader',
            options: {
              configFile: 'tsconfig.dest.json',
            },
          },
        ],
      },
    ],
  },
  output: {
    path: resolve(dirname(fileURLToPath(import.meta.url)), './dest'),
    filename: 'index.js',
  },
  plugins: [
    new webpack.DefinePlugin({
      'process.env': {
        NODE_ENV: JSON.stringify(argv.mode || 'production'),
      },
    }),
    new webpack.ProvidePlugin({ Buffer: ['buffer', 'Buffer'] }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: `${dirname(require.resolve(`@aztec/circuits.js`)).replace(
            /\/dest$/,
            '',
          )}/resources/aztec3-circuits.wasm`,
          to: 'aztec3-circuits.wasm',
        },
        {
          from: './src/index.html',
          to: 'index.html',
        },
      ],
    }),
  ],
  resolve: {
    plugins: [new ResolveTypeScriptPlugin()],
    alias: {
      // All node specific code, wherever it's located, should be imported as below.
      // Provides a clean and simple way to always strip out the node code for the web build.
      './node/index.js': false,
    },
    fallback: {
      crypto: false,
      os: false,
      fs: false,
      path: false,
      url: false,
      worker_threads: false,
      events: require.resolve('events/'),
      buffer: require.resolve('buffer/'),
      util: require.resolve('util/'),
      stream: require.resolve('stream-browserify'),
      string_decoder: require.resolve('string_decoder/'),
      tty: require.resolve('tty-browserify'),
    },
  },
  devServer: {
    port: 5173,
    historyApiFallback: true,
    client: {
      overlay: false,
    },
  },
});
