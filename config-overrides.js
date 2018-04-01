const rewireTypescript = require('react-app-rewire-typescript');

module.exports = function override(config, env) {
  return rewireTypescript(config, env);
};
