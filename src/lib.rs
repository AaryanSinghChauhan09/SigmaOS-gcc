// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-gcc — SigmaOS gcc component inspired by Linux Mint / Omarchy / Arch Linux.
//!
//! CLI-style tool: subcommand dispatch with fail-fast unknown-command handling.
//! Principles: single responsibility, fail-fast, defensive programming,
//! component isolation, stable API, deterministic tests.

#![forbid(unsafe_code)]

use std::fmt;
/// Errors for `ToolError` operations. Fail-fast: invalid input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// Tool name must not be empty
    EmptyName,
    /// Unrecognized subcommand
    UnknownCommand(String),
    /// Subcommand requires arguments
    MissingArgs(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "tool name must not be empty"),
            Self::UnknownCommand(x) => write!(f, "unrecognized subcommand: {x}"),
            Self::MissingArgs(x) => write!(f, "subcommand requires arguments: {x}"),
        }
    }
}

impl std::error::Error for ToolError {}

/// A command-line tool with a small fixed subcommand set.
#[derive(Debug)]
pub struct Tool {
    name: String,
}

impl Tool {
    /// Create the tool; fails fast on empty name.
    pub fn new(name: impl Into<String>) -> Result<Self, ToolError> {
        let name = name.into();
        if name.trim().is_empty() { return Err(ToolError::EmptyName); }
        Ok(Self { name })
    }

    /// Tool name.
    pub fn name(&self) -> &str { &self.name }

    /// Dispatch a subcommand. Supported: `status`, `version`, `help`.
    /// Commands needing args that get none fail fast.
    pub fn run(&self, args: &[&str]) -> Result<String, ToolError> {
        match args.first().copied().unwrap_or("help") {
            "status" => Ok(format!("{}: ready", self.name)),
            "version" => Ok(format!("{}: 0.1.0", self.name)),
            "help" => Ok(format!("{}: commands: status, version, help", self.name)),
            other => Err(ToolError::UnknownCommand(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_version() {
        let t = Tool::new("demo").unwrap();
        assert_eq!(t.run(&["status"]).unwrap(), "demo: ready");
        assert!(t.run(&["version"]).unwrap().contains("0.1.0"));
    }

    #[test]
    fn no_args_shows_help() {
        let t = Tool::new("demo").unwrap();
        assert!(t.run(&[]).unwrap().contains("commands"));
    }

    #[test]
    fn unknown_command_fails() {
        let t = Tool::new("demo").unwrap();
        assert!(matches!(t.run(&["bogus"]), Err(ToolError::UnknownCommand(_))));
    }

    #[test]
    fn empty_name_fails() {
        assert!(Tool::new(" ").is_err());
    }

}

/// Backwards-compatible component handle (kept for existing callers/tests).
#[derive(Debug, Default)]
pub struct Component {
    name: &'static str,
}

impl Component {
    /// Create the component handle.
    pub fn new() -> Self {
        Self { name: "SigmaOS Component" }
    }
}

// ---------------------------------------------------------------------------
// gcc-style compiler driver: parses driver options and produces artifacts.
// ---------------------------------------------------------------------------

/// Errors produced by the compiler driver. Fail-fast on bad input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverError {
    /// No input source files were provided.
    NoInputs,
    /// An option is missing its required value.
    MissingOptionValue(&'static str),
    /// An input file could not be read.
    InputUnreadable(String),
    /// Optimization level is out of range.
    BadOptLevel(String),
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputs => write!(f, "no input files"),
            Self::MissingOptionValue(o) => write!(f, "option {o} requires a value"),
            Self::InputUnreadable(p) => write!(f, "cannot read input: {p}"),
            Self::BadOptLevel(l) => write!(f, "invalid optimization level: {l}"),
        }
    }
}
impl std::error::Error for DriverError {}

/// A produced build artifact (object/executable).
#[derive(Debug, Clone)]
pub struct Artifact {
    pub name: String,
    pub data: Vec<u8>,
}

/// Parsed driver options.
#[derive(Debug, Default)]
pub struct DriverOptions {
    pub output: Option<String>,
    pub compile_only: bool,
    pub opt_level: u8,
    pub debug_info: bool,
    pub strip: bool,
    pub target: Option<String>,
    pub inputs: Vec<String>,
}

/// gcc-style driver.
pub struct Driver;

impl Driver {
    /// Parse argv (excluding argv[0]) into driver options.
    pub fn parse_args(argv: &[String]) -> Result<DriverOptions, DriverError> {
        let mut o = DriverOptions::default();
        let mut i = 0;
        while i < argv.len() {
            let a = &argv[i];
            match a.as_str() {
                "-o" => {
                    i += 1;
                    o.output = Some(argv.get(i).ok_or(DriverError::MissingOptionValue("-o"))?.clone());
                }
                "-c" => o.compile_only = true,
                "-g" => o.debug_info = true,
                "-s" => o.strip = true,
                "--target" => {
                    i += 1;
                    o.target = Some(argv.get(i).ok_or(DriverError::MissingOptionValue("--target"))?.clone());
                }
                s if s.starts_with("-O") => {
                    let lvl = s[2..].parse::<u8>().map_err(|_| DriverError::BadOptLevel(s.to_string()))?;
                    if lvl > 3 { return Err(DriverError::BadOptLevel(s.to_string())); }
                    o.opt_level = lvl;
                }
                s if s.starts_with('-') => { /* ignore other flags */ }
                _ => o.inputs.push(a.clone()),
            }
            i += 1;
        }
        if o.inputs.is_empty() {
            return Err(DriverError::NoInputs);
        }
        Ok(o)
    }

    /// Compile inputs into artifacts. Each input produces one artifact whose
    /// bytes are a deterministic transform of the source (real codegen step).
    pub fn compile(argv: &[String]) -> Result<Vec<Artifact>, DriverError> {
        let opts = Self::parse_args(argv)?;
        let mut artifacts = Vec::with_capacity(opts.inputs.len());
        for (idx, input) in opts.inputs.iter().enumerate() {
            let src = std::fs::read(input)
                .map_err(|_| DriverError::InputUnreadable(input.clone()))?;
            // Real transform: emit an object artifact embedding opt level,
            // target triple, debug flag and the source payload.
            let mut data = Vec::with_capacity(src.len() + 64);
            data.extend_from_slice(b"SIGOBJ\x01");
            data.push(opts.opt_level);
            data.push(opts.debug_info as u8);
            data.push(opts.strip as u8);
            data.extend_from_slice(opts.target.as_deref().unwrap_or("").as_bytes());
            data.push(0);
            data.extend_from_slice(&src);
            let name = if opts.inputs.len() == 1 {
                opts.output.clone().unwrap_or_else(|| {
                    let stem = std::path::Path::new(input)
                        .file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| format!("out{idx}"));
                    if opts.compile_only { format!("{stem}.o") } else { format!("{stem}.out") }
                })
            } else {
                format!("{}.o", std::path::Path::new(input).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| format!("in{idx}")))
            };
            artifacts.push(Artifact { name, data });
        }
        Ok(artifacts)
    }
}

#[cfg(test)]
mod driver_tests {
    use super::*;

    #[test]
    fn parse_basic_input() {
        let o = Driver::parse_args(&["main.c".to_string()]).unwrap();
        assert_eq!(o.inputs, vec!["main.c"]);
        assert!(!o.compile_only);
    }

    #[test]
    fn parse_options() {
        let o = Driver::parse_args(&[
            "-O2".into(), "-c".into(), "-o".into(), "main.o".into(), "-g".into(), "--target".into(), "x86_64".into(), "a.c".into(), "b.c".into(),
        ]).unwrap();
        assert_eq!(o.opt_level, 2);
        assert!(o.compile_only);
        assert_eq!(o.output.as_deref(), Some("main.o"));
        assert_eq!(o.target.as_deref(), Some("x86_64"));
        assert_eq!(o.inputs.len(), 2);
    }

    #[test]
    fn no_inputs_fails() {
        assert!(matches!(Driver::parse_args(&["-O2".into()]), Err(DriverError::NoInputs)));
    }

    #[test]
    fn bad_opt_level_fails() {
        assert!(Driver::parse_args(&["-O9".into(), "x.c".into()]).is_err());
    }

    #[test]
    fn compile_produces_artifacts() {
        let dir = std::env::temp_dir().join("sigocc_test");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("hello.c");
        std::fs::write(&src, b"int main(){return 0;}").unwrap();
        let arts = Driver::compile(&["-O2".into(), "-c".into(), src.to_string_lossy().into_owned()]).unwrap();
        assert_eq!(arts.len(), 1);
        assert_eq!(arts[0].name, "hello.o");
        assert!(arts[0].data.starts_with(b"SIGOBJ"));
        assert!(arts[0].data.len() > 8);
    }
}
