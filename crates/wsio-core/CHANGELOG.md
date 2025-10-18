# Changelog

# v0.1.0

[compare changes](https://github.com/ws-io/wsio-rs/compare/5b4d5d49...wsio-core-v0.1.0)

### 🚀 Enhancements

- Add initial structs following the same architecture as socket.io ([96f53b0](https://github.com/ws-io/wsio-rs/commit/96f53b0))
- Add initial config structure and apply it ([00c4d97](https://github.com/ws-io/wsio-rs/commit/00c4d97))
- Add packet structure and related codecs ([f09d100](https://github.com/ws-io/wsio-rs/commit/f09d100))
- Initial implementation of service struct handling for Tower service ([f06506d](https://github.com/ws-io/wsio-rs/commit/f06506d))
- Continue implementing core infrastructure ([f768fed](https://github.com/ws-io/wsio-rs/commit/f768fed))
- Enhance dispatch_request with header validation and WebSocket support ([1ce5056](https://github.com/ws-io/wsio-rs/commit/1ce5056))
- Implement WebSocket upgrade and namespace handler ([f8ee8ae](https://github.com/ws-io/wsio-rs/commit/f8ee8ae))
- Add namespace builder and support per-namespace packet codec configuration ([c65e4c4](https://github.com/ws-io/wsio-rs/commit/c65e4c4))
- Add context struct and with_auth method to WsIoNamespaceBuilder ([472f88f](https://github.com/ws-io/wsio-rs/commit/472f88f))
- Make WsIoContext public ([7df1ad9](https://github.com/ws-io/wsio-rs/commit/7df1ad9))
- Rename context to connection and generate SID on request to instantiate and register in namespace ([cdfb3bd](https://github.com/ws-io/wsio-rs/commit/cdfb3bd))
- Handle connection creation after on_upgrade and separate stream read/write ([530203f](https://github.com/ws-io/wsio-rs/commit/530203f))
- Add skip_serializing_none to WsIoPacket ([052c9b0](https://github.com/ws-io/wsio-rs/commit/052c9b0))
- Initial implementation of connection.init and fix packet codec decode issues with bincode/msgpack ([65da9f7](https://github.com/ws-io/wsio-rs/commit/65da9f7))
- Implement connection activation and cleanup on disconnect or termination ([f1c5a50](https://github.com/ws-io/wsio-rs/commit/f1c5a50))
- Complete ns.on_connect handling and connection management ([1a2026a](https://github.com/ws-io/wsio-rs/commit/1a2026a))
- Implement connection.on_disconnect functionality ([514ff04](https://github.com/ws-io/wsio-rs/commit/514ff04))

### 💅 Refactors

- Rename inner to runtime ([41597c4](https://github.com/ws-io/wsio-rs/commit/41597c4))
- Change all of the parameter type Cow<'static, str> to impl AsRef<str> ([5d40e27](https://github.com/ws-io/wsio-rs/commit/5d40e27))
- Rename rtio-server folder to wsio-server ([6f97f57](https://github.com/ws-io/wsio-rs/commit/6f97f57))
- Move packet module to core and rename WsIo to WsIoServer in all server modules ([e1aae62](https://github.com/ws-io/wsio-rs/commit/e1aae62))
- Rename builder.build_layer to build and add layer method on WsIo ([f875a95](https://github.com/ws-io/wsio-rs/commit/f875a95))
- Rename WsIoServer.ns to WsIoServer.new_namespace_builder and WsIoServerNamespaceBuilder.build to register ([9cd1db3](https://github.com/ws-io/wsio-rs/commit/9cd1db3))

### 🏡 Chore

- Add base files ([911e98f](https://github.com/ws-io/wsio-rs/commit/911e98f))
- Change package name to wsio ([acc7502](https://github.com/ws-io/wsio-rs/commit/acc7502))
- Tidy up code and temporarily implement WsIoService with placeholder logic ([6ca4e93](https://github.com/ws-io/wsio-rs/commit/6ca4e93))
- Remove WsIoService.runtime method ([18e421e](https://github.com/ws-io/wsio-rs/commit/18e421e))
- Rename package name to wsioxide ([78f2daf](https://github.com/ws-io/wsio-rs/commit/78f2daf))
- Set cargo include config ([cd95391](https://github.com/ws-io/wsio-rs/commit/cd95391))
- Set package desc ([6c814ef](https://github.com/ws-io/wsio-rs/commit/6c814ef))

### ❤️ Contributors

- Kiki-kanri
