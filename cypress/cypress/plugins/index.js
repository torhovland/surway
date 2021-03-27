const { cypressBrowserPermissionsPlugin } = require('cypress-browser-permissions')

module.exports = (on, config) => {
  // The plugin may modify the Cypress config, so be sure
  // to return it
  config = cypressBrowserPermissionsPlugin(on, config)

  //
  // Any existing plugins you are using
  //

  return config
}