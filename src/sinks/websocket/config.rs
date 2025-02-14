use codecs::JsonSerializerConfig;
use snafu::ResultExt;
use vector_config::configurable_component;

use crate::{
    codecs::EncodingConfig,
    config::{AcknowledgementsConfig, GenerateConfig, Input, SinkConfig, SinkContext},
    http::Auth,
    sinks::{
        websocket::sink::{ConnectSnafu, WebSocketConnector, WebSocketError, WebSocketSink},
        Healthcheck, VectorSink,
    },
    tls::{MaybeTlsSettings, TlsEnableableConfig},
};

/// Configuration for the `websocket` sink.
#[configurable_component(sink)]
#[derive(Clone, Debug)]
pub struct WebSocketSinkConfig {
    /// The WebSocket URI to connect to.
    ///
    /// This should include the protocol and host, but can also include the port, path, and any other valid part of a URI.
    pub uri: String,

    #[configurable(derived)]
    pub tls: Option<TlsEnableableConfig>,

    #[configurable(derived)]
    pub encoding: EncodingConfig,

    /// The interval, in seconds, between sending PINGs to the remote peer.
    pub ping_interval: Option<u64>,

    /// The timeout, in seconds, while waiting for a PONG response from the remote peer.
    ///
    /// If a response is not received in this time, the connection is reestablished.
    pub ping_timeout: Option<u64>,

    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    pub acknowledgements: AcknowledgementsConfig,

    #[configurable(derived)]
    pub auth: Option<Auth>,
}

impl GenerateConfig for WebSocketSinkConfig {
    fn generate_config() -> toml::Value {
        toml::Value::try_from(Self {
            uri: "ws://127.0.0.1:9000/endpoint".into(),
            tls: None,
            encoding: JsonSerializerConfig::new().into(),
            ping_interval: None,
            ping_timeout: None,
            acknowledgements: Default::default(),
            auth: None,
        })
        .unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "websocket")]
impl SinkConfig for WebSocketSinkConfig {
    async fn build(&self, _cx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        let connector = self.build_connector()?;
        let ws_sink = WebSocketSink::new(self, connector.clone())?;

        Ok((
            VectorSink::from_event_streamsink(ws_sink),
            Box::pin(async move { connector.healthcheck().await }),
        ))
    }

    fn input(&self) -> Input {
        Input::log()
    }

    fn sink_type(&self) -> &'static str {
        "websocket"
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}

impl WebSocketSinkConfig {
    fn build_connector(&self) -> Result<WebSocketConnector, WebSocketError> {
        let tls = MaybeTlsSettings::from_config(&self.tls, false).context(ConnectSnafu)?;
        WebSocketConnector::new(self.uri.clone(), tls, self.auth.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<WebSocketSinkConfig>();
    }
}
