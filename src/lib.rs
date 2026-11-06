//! SigmaOS-gcc — compiler driver for SigmaOS.
//!
//! Models the gcc driver pipeline: argument parsing → preprocessing →
//! compilation → assembly → linking, with per-target tool/spec selection.

pub mod driver;
pub mod flags;
pub mod pipeline;
pub mod target;

pub use driver::{CompileJob, Driver, DriverError};
pub use flags::{Flag, FlagKind, ParsedArgs};
pub use pipeline::{Artifact, ArtifactKind, Pipeline, Stage};
pub use target::{Abi, Target, TargetSpec};
