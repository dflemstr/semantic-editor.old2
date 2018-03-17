const {injectBabelPlugin, getLoader, getBabelLoader} = require('react-app-rewired');
const rewireTypescript = require('react-app-rewire-typescript');
const ModuleScopePlugin = require('react-dev-utils/ModuleScopePlugin');
const path = require('path');
const webpack = require('webpack');

const isLoader = name => rules => {
  return rules.loader && rules.loader.indexOf(name) >= 0;
};

module.exports = function override(config, env) {
  config = rewireTypescript(config, env, {
    appendTsSuffixTo: [/\.rs$/]
  });

  config = injectBabelPlugin(['import', {libraryName: 'antd', style: 'css'}], config);

  let fileLoader = getLoader(config.module.rules, isLoader('file-loader'));
  fileLoader.exclude.push(/\.rs$/);

  let babelLoader = getBabelLoader(config.module.rules);
  babelLoader.include = [babelLoader.include, path.resolve(babelLoader.include, '..', 'target')];
  babelLoader.test = [babelLoader.test, /\.rs$/];

  config.module.rules.push(
    {
      test: /\.rs$/,
      use: [{
        loader: 'ts-loader',
      }, {
        loader: 'rust-native-wasm-loader',
        options: {
          release: true,
          wasmBindgen: true,
          wasm2es6js: true,
          typescript: true,
        }
      }]
    }
  );

  const resolvePlugins = config.resolve.plugins;
  for (let i = 0; i < resolvePlugins.length; i++) {
    if (resolvePlugins[i] instanceof ModuleScopePlugin) {
      resolvePlugins.splice(i, 1);
      break;
    }
  }

  const node = config.node || (config.node = {});
  node.fs = 'empty';

  config.plugins.push(new webpack.WatchIgnorePlugin([/target/]));

  config.resolve.modules.push(path.resolve('./src'));

  return config;
};
