# Changelog

## [0.3.1](https://github.com/ws-io/ws.io-rs/compare/wsio-server-v0.3.0...wsio-server-v0.3.1) - 2025-10-22 05:54

### 🏀 Examples

- move files to `examples` workspace ([667bfe5](https://github.com/ws-io/ws.io-rs/commit/667bfe5))
- add disconnect example ([dacb448](https://github.com/ws-io/ws.io-rs/commit/dacb448))
- rename files ([cf536ad](https://github.com/ws-io/ws.io-rs/commit/cf536ad))

### 🏡 Chore

- disable or replace certain dependency features to reduce overall dependencies ([1d88ae3](https://github.com/ws-io/ws.io-rs/commit/1d88ae3))

### 💅 Refactors

- change all `status` fields to use `AtomicU8` and add operation lock for major actions like connect/disconnect ([5321b97](https://github.com/ws-io/ws.io-rs/commit/5321b97))
- *(server)* tidy up code ([2b7e382](https://github.com/ws-io/ws.io-rs/commit/2b7e382))
- *(server)* move `WsIoServerRuntime.handle_on_upgrade_request` into `WsIoServerNamespace` ([a7d1157](https://github.com/ws-io/ws.io-rs/commit/a7d1157))
- change return type of some `struct::new` functions to `Arc<Self>` ([a7ce497](https://github.com/ws-io/ws.io-rs/commit/a7ce497))
- *(server)* move spawn of request upgrade task in `dispatch_request` to runtime and register task in map for better management ([63bdb1a](https://github.com/ws-io/ws.io-rs/commit/63bdb1a))
- rename `xxxConnectionStatus` to `ConnectionStatus` ([3863d68](https://github.com/ws-io/ws.io-rs/commit/3863d68))
- update some format usage ([efdba68](https://github.com/ws-io/ws.io-rs/commit/efdba68))
- simplify status checking and transitions within `connection.close` ([d5c478e](https://github.com/ws-io/ws.io-rs/commit/d5c478e))
- replace `match` statements for extracting and converting `Option` values with chained `map` and `transpose` calls ([cf7f9b3](https://github.com/ws-io/ws.io-rs/commit/cf7f9b3))

### 🚀 Enhancements

- *(server)* add `WsIoServerConnectionExtensions` ([3dca472](https://github.com/ws-io/ws.io-rs/commit/3dca472))
- allow configuration of WebSocket settings such as `max_frame_size` ([0b2b491](https://github.com/ws-io/ws.io-rs/commit/0b2b491))
- *(server)* add `WsIoServerConnection.emit` method ([f6ff682](https://github.com/ws-io/ws.io-rs/commit/f6ff682))
- *(server)* add `WsIoServerConnection.spawn_task` method ([8a25fcf](https://github.com/ws-io/ws.io-rs/commit/8a25fcf))

## [0.3.0](https://github.com/ws-io/ws.io-rs/compare/wsio-server-v0.2.1...wsio-server-v0.3.0) - 2025-10-20 17:48

### 🏀 Examples

- add client and server examples ([88a2fce](https://github.com/ws-io/ws.io-rs/commit/88a2fce))
- *(server)* add examples file ([ff1444b](https://github.com/ws-io/ws.io-rs/commit/ff1444b))

### 🏡 Chore

- *(server)* reduce default `auth_timeout` duration ([3bf00cd](https://github.com/ws-io/ws.io-rs/commit/3bf00cd))

### 💅 Refactors

- update `handle_incoming_packet` to require successful decoding before processing; return error to upper layer and exit `read_ws_stream_task` on failure ([76bf3dd](https://github.com/ws-io/ws.io-rs/commit/76bf3dd))
- *(server)* tidy up code ([90f94c5](https://github.com/ws-io/ws.io-rs/commit/90f94c5))
- tidy up code ([4e5a362](https://github.com/ws-io/ws.io-rs/commit/4e5a362))
- change `Connection` message `tx/rx` from `unbounded_channel` to bounded `channel` ([4e6a130](https://github.com/ws-io/ws.io-rs/commit/4e6a130))
- *(server)* remove `WsIoServerConnection.on` method and related code ([dfe85d8](https://github.com/ws-io/ws.io-rs/commit/dfe85d8))
- *(server)* remove unnecessary state checks ([153a9f4](https://github.com/ws-io/ws.io-rs/commit/153a9f4))

### 🚀 Enhancements

- add cbor packet codec ([f3e1fa9](https://github.com/ws-io/ws.io-rs/commit/f3e1fa9))
- *(server)* add `cancel_token` getter to `WsIoServerConnection` and invoke `cancel_token.cancel` in `cleanup` ([3b1076d](https://github.com/ws-io/ws.io-rs/commit/3b1076d))
- *(server)* add `namespace_count` method ([51d4867](https://github.com/ws-io/ws.io-rs/commit/51d4867))

### 🩹 Fixes

- *(server)* resolve status deadlock issue in `connection.handle_auth_packet` ([a36b54b](https://github.com/ws-io/ws.io-rs/commit/a36b54b))

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
