pub mod db;
pub mod embedding;
pub mod mcp;
pub mod render;
pub mod services;

/// Test utilities for unit and integration testing.
/// Only available with cfg(test) or feature "testing".
#[cfg(any(test, feature = "testing"))]
pub mod testing;
