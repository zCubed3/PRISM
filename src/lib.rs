#![allow(clippy::needless_return)]
#![allow(dead_code)]

//! # Prism
//!   CPU Compute library for Rust
//!
//!   Built with user extensibility in mind!

/// Various performance measuring tools
pub mod perf;

/// Compute section of the library (prelude includes this)
pub mod compute;

/// Contains the most commonly used parts of the rendering library
pub mod prelude {
    pub use rgml::prelude::*;
    pub use rgml::real::RealNumber;

    pub use crate::compute::buffer::*;
    pub use crate::compute::dispatcher::*;
    pub use crate::compute::kernel::*;
    pub use crate::compute::msaa::*;
}
