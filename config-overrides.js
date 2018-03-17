const {injectBabelPlugin} = require('react-app-rewired');
const rewireTypescript = require('react-app-rewire-typescript');

module.exports = function override(config, env) {
  config = rewireTypescript(config, env);
  config = injectBabelPlugin(['import', {libraryName: 'antd', style: 'css'}], config);
  return config;
};
