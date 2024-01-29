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
  devtool: 'source-map',
  entry: {
    main: './src/index.ts',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
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
          from: './src/index.html',
          to: 'index.html',
        },
      ],
    }),
  ],
  resolve: {
    plugins: [new ResolveTypeScriptPlugin()],
    // alias: {
    //   // All node specific code, wherever it's located, should be imported as below.
    //   // Provides a clean and simple way to always strip out the node code for the web build.
    //   './node/index.js': false,
    // },
    // TODO: Get rid of these polyfills! Properly abstract away node/browser differences and bundle aztec.js properly.
    // Consumers of our project should not have to jump through hoops to use it.
    // (Fairly sure the false crypto here means things will break.)
    fallback: {
      crypto: false,
      fs: false,
      path: false,
      events: require.resolve('events/'),
      stream: require.resolve('stream-browserify'),
      tty: require.resolve('tty-browserify'),
      util: require.resolve('util/'),
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
