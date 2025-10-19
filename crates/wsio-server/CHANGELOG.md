# Changelog

## [0.1.3](https://github.com/ws-io/ws.io-rs/compare/wsio-server-v0.1.2...wsio-server-v0.1.3) - 2025-10-19 03:28

### 💅 Refactors

- *(server)* change `connection.on` method event parameter type to `impl AsRef<str>` ([acb8e50](https://github.com/ws-io/ws.io-rs/commit/acb8e50))

## [0.1.2](https://github.com/ws-io/ws.io-rs/compare/wsio-server-v0.1.1...wsio-server-v0.1.2) - 2025-10-19 00:35

### 🏡 Chore

- lint code ([945b186](https://github.com/ws-io/ws.io-rs/commit/945b186))

### 💅 Refactors

- *(server)* update `namespace builder.with_auth` to change handler `data` parameter to `Option<&D>` ([61179f7](https://github.com/ws-io/ws.io-rs/commit/61179f7))

### 🚀 Enhancements

- *(server)* add namespace middleware functionality ([4893bbc](https://github.com/ws-io/ws.io-rs/commit/4893bbc))
- *(server)* add connection.server method ([44d4c46](https://github.com/ws-io/ws.io-rs/commit/44d4c46))
- add `connection.on` method to register event handlers ([3e352f6](https://github.com/ws-io/ws.io-rs/commit/3e352f6))

## [0.1.1](https://github.com/ws-io/ws.io-rs/compare/wsio-server-v0.1.0...wsio-server-v0.1.1) - 2025-10-18 14:57

### 💅 Refactors

- *(server)* lint code ([b97bdda](https://github.com/ws-io/ws.io-rs/commit/b97bdda))

### 🚀 Enhancements

- *(server)* implement connection handling for `on_message` and `auth` packet reception ([3aefaab](https://github.com/ws-io/ws.io-rs/commit/3aefaab))
