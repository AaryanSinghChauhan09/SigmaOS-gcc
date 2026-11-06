//! Compilation pipeline — the stages a driver runs in order.

use crate::flags::ParsedArgs;
use crate::target::TargetSpec;

/// Kind of artifact a stage emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    /// Preprocessed source.
    Preprocessed,
    /// Assembly text.
    Assembly,
    /// Object file.
    Object,
    /// Linked executable/library.
    Linked,
}

/// Output of one pipeline stage.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub name: String,
    pub data: Vec<u8>,
}

/// Pipeline stages in gcc order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Preprocess,
    Compile,
    Assemble,
    Link,
}

/// Ordered list of stages for an invocation.
pub fn plan_stages(args: &ParsedArgs) -> Vec<Stage> {
    let mut stages = Vec::new();
    if args.preprocess_only {
        stages.push(Stage::Preprocess);
    } else {
        stages.push(Stage::Preprocess);
        stages.push(Stage::Compile);
        if args.assemble_only {
            stages.push(Stage::Assemble);
        } else {
            stages.push(Stage::Assemble);
            if args.link {
                stages.push(Stage::Link);
            }
        }
    }
    stages
}

/// Simulated pipeline executor — produces deterministic placeholder artifacts
/// recording which stages ran (real cc1/as/ld invocation is out of scope for
/// the driver model; this validates ordering + spec propagation).
pub struct Pipeline;

impl Pipeline {
    /// Run the planned stages, returning the artifacts produced.
    pub fn run(args: &ParsedArgs, spec: &TargetSpec) -> Result<Vec<Artifact>, String> {
        if args.inputs.is_empty() {
            return Err("no input files".to_string());
        }
        let stages = plan_stages(args);
        let mut out = Vec::new();
        for stage in stages {
            let kind = match stage {
                Stage::Preprocess => ArtifactKind::Preprocessed,
                Stage::Compile => ArtifactKind::Assembly,
                Stage::Assemble => ArtifactKind::Object,
                Stage::Link => ArtifactKind::Linked,
            };
            let name = args
                .output
                .clone()
                .unwrap_or_else(|| format!("{}.{:?}", args.inputs[0], kind).to_lowercase());
            // Deterministic content: stage + input + target triple.
            let mut data = format!("stage={stage:?} input={} target={}-{}-{} opt={}",
                args.inputs[0], spec.target.arch, spec.target.os, spec.target.env, spec.opt_level)
                .into_bytes();
            if spec.debug {
                data.extend_from_slice(b" debug=1");
            }
            out.push(Artifact { kind, name, data });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::parse_args;
    use crate::target::Target;

    #[test]
    fn stage_planning() {
        let p = parse_args(&["-c".into(), "x.c".into()]);
        assert_eq!(plan_stages(&p), vec![Stage::Preprocess, Stage::Compile, Stage::Assemble]);
        let p = parse_args(&["-E".into(), "x.c".into()]);
        assert_eq!(plan_stages(&p), vec![Stage::Preprocess]);
        let p = parse_args(&["x.c".into()]);
        assert_eq!(plan_stages(&p), vec![Stage::Preprocess, Stage::Compile, Stage::Assemble, Stage::Link]);
    }

    #[test]
    fn pipeline_requires_input() {
        let args = parse_args(&[]);
        let spec = TargetSpec::for_target(Target::host(), false, false, false, 0);
        assert!(Pipeline::run(&args, &spec).is_err());
    }
}
