pub mod config;
pub mod request_builder;
pub mod service;
pub mod sink;

#[cfg(test)]
mod tests;

pub use self::config::DatadogEventsConfig;

use crate::config::SinkDescription;

inventory::submit! {
    SinkDescription::new::<DatadogEventsConfig>("datadog_events")
}
