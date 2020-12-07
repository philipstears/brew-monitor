//! This crate provides facilities for communicating with a Grainfather controller.
//!
//! There are broadly 3 components to this library, protocol parsing/unparsing,
//! bluetooth helpers, and an implementation of a client using the btleplug
//! bluetooth library.
//!
//! [Command](crate::Command), [Notification](crate::Notification), and
//! [Recipe](crate::Recipe) are the three principal protocol-level types
//! and support the parsing/unparsing of those entities.
//!
//! The bluetooth helpers include the [service id](crate::SERVICE_ID), and
//! [read](crate::CHARACTERISTIC_ID_READ)/[write](crate::CHARACTERISTIC_ID_WRITE)
//! characteristic ids, as well as a [helper](crate::has_grainfather_service_id) that
//! can be used to see if the service id is contained within an Extended Information
//! Report returned by a bluetooth library.
//!
//! Finally, there's a `btleplug` feature (turned on by default), which makes
//! a [client](crate::btleplug::Client) available which wraps a
//! [`btleplug::api::Peripheral`](::btleplug::api::Peripheral) and exposes
//! an API for interacting with the Grainfather Controller in terms of commands,
//! notifications, and recipes.

// TODO: review temperature units
pub mod calc;

mod proto;
pub use proto::*;

mod bluetooth;
pub use bluetooth::*;
