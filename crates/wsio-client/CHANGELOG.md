# Changelog

## [0.2.2](https://github.com/ws-io/ws.io-rs/compare/wsio-client-v0.2.1...wsio-client-v0.2.2) - 2025-10-22 05:54

### 🏀 Examples

- move files to `examples` workspace ([667bfe5](https://github.com/ws-io/ws.io-rs/commit/667bfe5))
- add disconnect example ([dacb448](https://github.com/ws-io/ws.io-rs/commit/dacb448))
- add `connection_stress` client example ([61719a0](https://github.com/ws-io/ws.io-rs/commit/61719a0))
- rename files ([cf536ad](https://github.com/ws-io/ws.io-rs/commit/cf536ad))

### 🏡 Chore

- *(client)* format `Cargo.toml` ([03bd228](https://github.com/ws-io/ws.io-rs/commit/03bd228))
- disable or replace certain dependency features to reduce overall dependencies ([1d88ae3](https://github.com/ws-io/ws.io-rs/commit/1d88ae3))

### 💅 Refactors

- *(client)* change `WsIoClientRuntime.connection` to `ArcSwapOption<WsIoClientConnection>` ([e1576a2](https://github.com/ws-io/ws.io-rs/commit/e1576a2))
- change all `status` fields to use `AtomicU8` and add operation lock for major actions like connect/disconnect ([5321b97](https://github.com/ws-io/ws.io-rs/commit/5321b97))
- change return type of some `struct::new` functions to `Arc<Self>` ([a7ce497](https://github.com/ws-io/ws.io-rs/commit/a7ce497))
- rename `xxxConnectionStatus` to `ConnectionStatus` ([3863d68](https://github.com/ws-io/ws.io-rs/commit/3863d68))
- update some format usage ([efdba68](https://github.com/ws-io/ws.io-rs/commit/efdba68))
- simplify status checking and transitions within `connection.close` ([d5c478e](https://github.com/ws-io/ws.io-rs/commit/d5c478e))
- replace `match` statements for extracting and converting `Option` values with chained `map` and `transpose` calls ([cf7f9b3](https://github.com/ws-io/ws.io-rs/commit/cf7f9b3))

### 🚀 Enhancements

- *(client)* add clone derive to `WsIoClient` ([53c3476](https://github.com/ws-io/ws.io-rs/commit/53c3476))
- allow configuration of WebSocket settings such as `max_frame_size` ([0b2b491](https://github.com/ws-io/ws.io-rs/commit/0b2b491))
- *(client)* handle disconnect packet ([4da8353](https://github.com/ws-io/ws.io-rs/commit/4da8353))

### 🩹 Fixes

- *(client)* ensure `disconnect` immediately breaks `run_connection_loop` even if it's sleeping ([0f4a780](https://github.com/ws-io/ws.io-rs/commit/0f4a780))
- *(client)* normalize multiple consecutive slashes in URL namespace to a single slash ([0322671](https://github.com/ws-io/ws.io-rs/commit/0322671))
- *(client)* resolve issue where leading `/` in connection URL path caused connection failure ([fa5ca8c](https://github.com/ws-io/ws.io-rs/commit/fa5ca8c))

## [0.2.1](https://github.com/ws-io/ws.io-rs/compare/wsio-client-v0.2.0...wsio-client-v0.2.1) - 2025-10-20 17:48

### 🏀 Examples

- add client and server examples ([88a2fce](https://github.com/ws-io/ws.io-rs/commit/88a2fce))

### 💅 Refactors

- update `handle_incoming_packet` to require successful decoding before processing; return error to upper layer and exit `read_ws_stream_task` on failure ([76bf3dd](https://github.com/ws-io/ws.io-rs/commit/76bf3dd))
- tidy up code ([4e5a362](https://github.com/ws-io/ws.io-rs/commit/4e5a362))
- change `Connection` message `tx/rx` from `unbounded_channel` to bounded `channel` ([4e6a130](https://github.com/ws-io/ws.io-rs/commit/4e6a130))
- *(client)* rename `namespace_url` to `url` ([97e7675](https://github.com/ws-io/ws.io-rs/commit/97e7675))
- *(client)* move `connection.init` call in `run_connection` to occur before spawning read/write tasks ([0fcf536](https://github.com/ws-io/ws.io-rs/commit/0fcf536))
- *(client)* rename `WsIoClientBuilder.on_ready` to `on_connection_ready` ([ed0c7ca](https://github.com/ws-io/ws.io-rs/commit/ed0c7ca))

### 🚀 Enhancements

- add cbor packet codec ([f3e1fa9](https://github.com/ws-io/ws.io-rs/commit/f3e1fa9))
- *(client)* allow custom configuration of `init_timeout`, `ready_timeout`, and `reconnection_delay` ([161e055](https://github.com/ws-io/ws.io-rs/commit/161e055))
- *(client)* add `WsIoClientBuilder.on_connection_close` method and invoke it inside `connection.cleanup` ([7f8fb23](https://github.com/ws-io/ws.io-rs/commit/7f8fb23))
- *(client)* add `on_ready` method to builder and invoke configured handler after connection transitions to `ready` state ([167d618](https://github.com/ws-io/ws.io-rs/commit/167d618))

## [0.2.0](https://github.com/ws-io/ws.io-rs/compare/wsio-client-v0.1.1...wsio-client-v0.2.0) - 2025-10-20 05:35

### 💅 Refactors

- *(client)* rename `WsIoClientConfig.auth` to `auth_handler` ([0a73a04](https://github.com/ws-io/ws.io-rs/commit/0a73a04))

### 🚀 Enhancements

- *(client)* implement connection establishment with init/ready packet handling and add connection close/cleanup functionality ([28bb1a1](https://github.com/ws-io/ws.io-rs/commit/28bb1a1))

### 🩹 Fixes

- add missing Tokio features ([0fa2c13](https://github.com/ws-io/ws.io-rs/commit/0fa2c13))

## [0.1.1](https://github.com/ws-io/ws.io-rs/compare/wsio-client-v0.1.0...wsio-client-v0.1.1) - 2025-10-19 18:40

### 🚀 Enhancements

- *(client)* add empty `WsIoClientConnection` struct ([78df31b](https://github.com/ws-io/ws.io-rs/commit/78df31b))
- *(client)* add namespace url and auth configs ([6934beb](https://github.com/ws-io/ws.io-rs/commit/6934beb))
- *(client)* add base config, builder and runtime files ([859e39a](https://github.com/ws-io/ws.io-rs/commit/859e39a))

## [0.1.0](https://github.com/ws-io/ws.io-rs/releases/tag/wsio-client-v0.1.0) - 2025-10-19 03:28

### 🏡 Chore

- *(client)* add base files ([a70927d](https://github.com/ws-io/ws.io-rs/commit/a70927d))
