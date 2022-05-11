# `get-random-emoji` [![Build Status](https://travis-ci.com/lukewhrit/random-emoji.svg?branch=master)](https://travis-ci.com/lukewhrit/random-emoji)

> Simple module to get random emojis strings.

## Install

```
$ npm install get-random-emoji
```

## Usage

```js
const getEmoji = require('get-random-emoji')

getEmoji()
//=> '😀'
```

## API

### getEmoji()

Returns a pseudo-random emoji from the list defined in [`emojis.js`](./emojis.js).
