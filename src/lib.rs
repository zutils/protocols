//! # What is it?
//! The protocols crate is native rust library to make it easy to send messages between systems.  
//! 
//! The purpose of this crate at the moment is to let the community know what work is being done.  At the moment, the protocols library is not ready for commercial use, and any included protocols should not be considered standard until a system of upgradability has been determined. 
//! 
//! Check out the examples folder in the repository for basic usage.
//! 
//! # How it works
//! Each message sent uses a root-message (defined in this crate) whose purpose is to match the data in the packet to a schema.  If the receiver supports decoding of a specific schema, it will continue with the handling process.  
//! 
//! At the moment, Windows DLL files are dynamically loaded from both plugin folder, and at initialization from ./libraries/<your-library>/target/debug/*.dll .  If you use the create-protocols-plugin cargo application in the libraries folder, the dll from "cargo build" should be ready for loading immediately.
//! 
//! # Contributing:
//!  I welcome any contrubutions, and at the moment.  Due to an initial load of technical debt, core design is especially in need of ideas and recommendations.  Contact me via Github.
//! 
//! # Problems still dealing with:
//!  How do the different plugins talk with each other?  Do they need to talk to eachother?
//!  Currently, the root-message protocol uses Protobufs, but that may change based on need in the future.  
//!  
//! 
//! # Future:
//!  Language Agnostic.  Protocols needs FFI capability and to support as many languages as possible.
//! 
//!  Cross Platform. This crate makes little sense if it doesn't work with other operating systems.
//!  
//! Standards. An existing set of "Standards" will be introduced as a separate crate. You should be able to create your own system of standards as well.  
//!  
//! Dockers.  Handling of each message type should happen in a contained system for security and possibly expandability.
//!  
//! Standard Security Layers. (still planning) It may be important to know if the messages have been tampered with or replayed.
//! 
//! # What this crate is NOT:
//!  A replacement for existing protocols.  HTTP, SSH, SMTP, etc... are fine the way they are.  Althought creating a library for these crates wouldn't be too challenging, convincing webservers and browsers of all types to support the protocols crate would be considered an issue.
//! 
//! # Installation
//! Add this to cargo.toml
//! 
//! ```text
//! [dependencies]
//! plugins = "*"
//! ```

#![feature(as_cell)]

pub mod transporter;
pub mod autogen_protobuf;
pub mod common;
pub mod hashenabler;
pub mod logging;
pub mod transport_glue;
pub mod commonlibrary;
pub mod pluginhandler;

#[cfg(not(target_arch = "wasm32"))]
pub mod wasmhandler;

#[cfg(not(target_arch = "wasm32"))]
pub mod buildfunctions;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::pluginhandler::{PluginHandler};

pub use crate::transporter::{Transporter, RootTransporter};
pub use crate::transport_glue::{TransportToModelGlue, TransportToProcessorGlue};
pub use crate::common::{CommonModelFunctions, CommonStructureFunctions, Modifiable};
pub use crate::autogen_protobuf::transport::*;

