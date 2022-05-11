'use strict'

const emojis = require('./emojis')

/**
 * Picks a random emoji from list and returns one.
 */
module.exports = () => {
  const num = Math.floor(Math.random() * emojis.length)

  return emojis[num]
}
