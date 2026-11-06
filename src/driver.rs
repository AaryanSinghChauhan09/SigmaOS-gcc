//! The gcc driver: argv → ParsedArgs → TargetSpec → Pipeline → artifacts.

use crate::flags::{parse_args, ParsedArgs};
use crate::pipeline::{Artifact, Pipeline};
use crate::target::{Target, TargetSpec};
use std::fmt;

/// Errors the driver can report before invoking the pipeline.
#[derive(Debug)]
pub enum DriverError {
    /// `--target` triple was not recognised.
    BadTarget(String),
    /// Pipeline-level failure (propagated as a string).
    Pipeline(String),
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadTarget(t) => write!(f, "unrecognised target triple: {t}"),
            Self::Pipeline(m) => write!(f, "pipeline error: {m}"),
        }
    }
}

impl std::error::Error for DriverError {}

/// A fully-resolved compile job.
#[derive(Debug)]
pub struct CompileJob {
    pub args: ParsedArgs,
    pub spec: TargetSpec,
}

/// The driver itself — stateless, like `gcc` the binary.
pub struct Driver;

impl Driver {
    /// Resolve argv into a job (parses flags + selects target spec).
    pub fn plan(argv: &[String]) -> Result<CompileJob, DriverError> {
        let args = parse_args(argv);
        let target = match &args.target {
            Some(t) => Target::parse(t).ok_or_else(|| DriverError::BadTarget(t.clone()))?,
            None => Target::host(),
        };
        let spec = TargetSpec::for_target(
            target,
            /*pic*/ true,
            /*pie*/ args.link,
            args.debug,
            args.opt_level,
        );
        Ok(CompileJob { args, spec })
    }

    /// Plan and run the pipeline for argv.
    pub fn compile(argv: &[String]) -> Result<Vec<Artifact>, DriverError> {
        let job = Self::plan(argv)?;
        Pipeline::run(&job.args, &job.spec).map_err(DriverError::Pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_compile() {
        let arts = Driver::compile(&["-O2".into(), "main.c".into(), "-o".into(), "main".into()])
            .unwrap();
        assert_eq!(arts.len(), 4);
        assert_eq!(arts.last().unwrap().kind, crate::pipeline::ArtifactKind::Linked);
        assert_eq!(arts.last().unwrap().name, "main");
    }

    #[test]
    fn rejects_bad_target() {
        assert!(Driver::compile(&["--target".into(), "mips-linux".into(), "x.c".into()]).is_err());
    }
}
