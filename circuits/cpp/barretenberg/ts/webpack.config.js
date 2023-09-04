import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import ResolveTypeScriptPlugin from 'resolve-typescript-plugin';
import webpack from 'webpack';

export default {
  target: 'web',
  mode: 'production',
  // Useful for debugging.
  // mode: 'development',
  // devtool: 'source-map',
  entry: './src/index.ts',
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: 'asset/inline',
      },
      {
        test: /\.worker\.ts$/,
        loader: 'worker-loader',
        options: { inline: 'no-fallback' },
      },
      {
        test: /\.tsx?$/,
        use: [
          {
            loader: 'ts-loader',
            options: { configFile: 'tsconfig.browser.json', onlyCompileBundledFiles: true },
          },
        ],
      },
    ],
  },
  output: {
    path: resolve(dirname(fileURLToPath(import.meta.url)), './dest/browser'),
    filename: 'index.js',
    module: true,
    library: {
      type: 'module',
    },
  },
  experiments: {
    outputModule: true,
  },
  plugins: [
    new webpack.DefinePlugin({ 'process.env.NODE_DEBUG': false }),
    new webpack.ProvidePlugin({ Buffer: ['buffer', 'Buffer'] }),
    new webpack.NormalModuleReplacementPlugin(/\/node\/(.*)\.js$/, function (resource) {
      resource.request = resource.request.replace('/node/', '/browser/');
    }),
  ],
  resolve: {
    plugins: [new ResolveTypeScriptPlugin()],
  },
  devServer: {
    hot: false,
    client: {
      logging: 'none',
      overlay: false,
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
};
