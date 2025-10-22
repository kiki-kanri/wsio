# Changelog

## [0.3.1](https://github.com/ws-io/ws.io-rs/compare/wsio-core-v0.3.0...wsio-core-v0.3.1) - 2025-10-22 05:54

### 🎨 Styles

- *(core)* format code ([af9329d](https://github.com/ws-io/ws.io-rs/commit/af9329d))

### 💅 Refactors

- change all `status` fields to use `AtomicU8` and add operation lock for major actions like connect/disconnect ([5321b97](https://github.com/ws-io/ws.io-rs/commit/5321b97))

## [0.3.0](https://github.com/ws-io/ws.io-rs/compare/wsio-core-v0.2.0...wsio-core-v0.3.0) - 2025-10-20 17:48

### 💅 Refactors

- *(core)* serialize and deserialize `WsIoPacketType` as numeric values instead of stringified numbers ([0112ebb](https://github.com/ws-io/ws.io-rs/commit/0112ebb))

### 🚀 Enhancements

- add cbor packet codec ([f3e1fa9](https://github.com/ws-io/ws.io-rs/commit/f3e1fa9))

## [0.2.0](https://github.com/ws-io/ws.io-rs/compare/wsio-core-v0.1.1...wsio-core-v0.2.0) - 2025-10-19 18:40

### 💅 Refactors

- major code overhaul ([09c6773](https://github.com/ws-io/ws.io-rs/commit/09c6773))
- remove functionality that sends codec type data after connection establishment ([f8190ff](https://github.com/ws-io/ws.io-rs/commit/f8190ff))

## [0.1.1](https://github.com/ws-io/ws.io-rs/compare/wsio-core-v0.1.0...wsio-core-v0.1.1) - 2025-10-19 00:35

### 🏡 Chore

- lint code ([945b186](https://github.com/ws-io/ws.io-rs/commit/945b186))

### 🚀 Enhancements

- add `connection.on` method to register event handlers ([3e352f6](https://github.com/ws-io/ws.io-rs/commit/3e352f6))
