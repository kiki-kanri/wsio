# Changelog

## [0.2.1](https://github.com/ws-io/ws.io-rs/compare/wsio-server-v0.2.0...wsio-server-v0.2.1) - 2025-10-20 05:35

### 💅 Refactors

- *(server)* refine error messages and prevent conflicting state transitions in `conn.handle_auth_packet` ([12feb87](https://github.com/ws-io/ws.io-rs/commit/12feb87))
- *(server)* extract `auth` match block in `connection.handle_incoming_packet` into a separate method and modify packet type handling to close connection on handler error ([f46f73c](https://github.com/ws-io/ws.io-rs/commit/f46f73c))
- *(server)* rename `cleanup_connection` to `remove_connection` ([8ebde37](https://github.com/ws-io/ws.io-rs/commit/8ebde37))
- *(server)* move all type handlers into their respective individual files instead of defining them in `types/file` ([9d3e9d5](https://github.com/ws-io/ws.io-rs/commit/9d3e9d5))

### 🚀 Enhancements

- *(server)* add `connection.off` method ([9a2ceac](https://github.com/ws-io/ws.io-rs/commit/9a2ceac))

### 🩹 Fixes

- add missing Tokio features ([0fa2c13](https://github.com/ws-io/ws.io-rs/commit/0fa2c13))

## [0.2.0](https://github.com/ws-io/ws.io-rs/compare/wsio-server-v0.1.3...wsio-server-v0.2.0) - 2025-10-19 18:40

### 🏡 Chore

- *(server)* mark some functions is inline ([b238875](https://github.com/ws-io/ws.io-rs/commit/b238875))
- *(server)* tidy up code ([4f7d5e5](https://github.com/ws-io/ws.io-rs/commit/4f7d5e5))

### 💅 Refactors

- major code overhaul ([09c6773](https://github.com/ws-io/ws.io-rs/commit/09c6773))
- *(server)* use `= Some(...)` instead of `.replace(...)` when setting optional configuration values ([2feb398](https://github.com/ws-io/ws.io-rs/commit/2feb398))
- remove functionality that sends codec type data after connection establishment ([f8190ff](https://github.com/ws-io/ws.io-rs/commit/f8190ff))

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
