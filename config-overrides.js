const rewireTypescript = require('react-app-rewire-typescript')
const {getLoader, loaderNameMatches} = require('react-app-rewired')

module.exports = function override (config, env) {
  const newConfig = rewireTypescript(config, env)
  const fileLoader = getLoader(newConfig.module.rules, r => loaderNameMatches(r, 'file-loader'))

  newConfig.resolve.extensions.push('.wasm')
  fileLoader.exclude.push(/\.wasm$/)
  config.output.webassemblyModuleFilename = 'static/wasm/[modulehash:8].wasm'

  return newConfig
}
