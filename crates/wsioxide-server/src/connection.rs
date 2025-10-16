use http::HeaderMap;

pub struct WsIoServerConnection {
    authorized: bool,
    headers: HeaderMap,
    sid: String,
}

impl WsIoServerConnection {
    pub(crate) fn new(sid: String, headers: HeaderMap) -> Self {
        Self {
            authorized: false,
            sid,
            headers,
        }
    }

    // Public methods

    #[inline]
    pub fn sid(&self) -> &str {
        &self.sid
    }
}
