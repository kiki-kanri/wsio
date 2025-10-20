# Changelog

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
