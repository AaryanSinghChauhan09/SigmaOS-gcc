//! Command-line flag parsing — the subset of gcc options the driver honours.

/// Category of a recognised flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagKind {
    Input,
    Output,
    Target,
    Optimization,
    Debug,
    Link,
    PreprocessOnly,
    CompileOnly,
    AssembleOnly,
    Help,
    Version,
    Unknown,
}

/// A single parsed flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    pub kind: FlagKind,
    pub name: String,
    pub value: Option<String>,
}

/// Result of parsing argv.
#[derive(Debug, Default)]
pub struct ParsedArgs {
    pub flags: Vec<Flag>,
    pub inputs: Vec<String>,
    pub output: Option<String>,
    pub target: Option<String>,
    pub opt_level: u8,
    pub debug: bool,
    pub link: bool,
    pub preprocess_only: bool,
    pub compile_only: bool,
    pub assemble_only: bool,
    pub help: bool,
    pub version: bool,
}

/// Parse gcc-style argv. Unknown flags are recorded, never fatal.
pub fn parse_args(argv: &[String]) -> ParsedArgs {
    let mut out = ParsedArgs::default();
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        if let Some(rest) = a.strip_prefix("--") {
            match rest {
                "help" => { out.help = true; flag(&mut out, "help", FlagKind::Help, None); }
                "version" => { out.version = true; flag(&mut out, "version", FlagKind::Version, None); }
                "target" => { i += 1; let v = argv.get(i).cloned(); flag(&mut out, "target", FlagKind::Target, v.clone()); out.target = v; }
                "param" | "specs" | "sysroot" => { i += 1; }
                _ => flag(&mut out, rest, FlagKind::Unknown, None),
            }
        } else if let Some(rest) = a.strip_prefix('-') {
            if rest.is_empty() {
                out.inputs.push(a.clone()); // "-" = stdin marker, treat as input
            } else {
                match rest {
                    "E" => out.preprocess_only = true,
                    "c" => out.compile_only = true,
                    "S" => out.preprocess_only = true,
                    "s" => out.assemble_only = true,
                    _ => flag(&mut out, rest, classify(rest), None),
                }
                if rest == "o" {
                    i += 1;
                    let v = argv.get(i).cloned();
                    out.output = v.clone();
                    flag(&mut out, "o", FlagKind::Output, v);
                }
                if let Some(l) = rest.strip_prefix('O') {
                    if let Ok(n) = l.parse::<u8>() { out.opt_level = n; }
                    flag(&mut out, rest, FlagKind::Optimization, None);
                } else if rest == "g" {
                    out.debug = true;
                    flag(&mut out, "g", FlagKind::Debug, None);
                } else if rest == "shared" || rest == "static" {
                    flag(&mut out, rest, FlagKind::Link, None);
                } else if rest == "l" {
                    i += 1;
                    flag(&mut out, "l", FlagKind::Link, argv.get(i).cloned());
                }
            }
        } else {
            out.inputs.push(a.clone());
        }
        i += 1;
    }
    out.link = !out.preprocess_only && !out.compile_only && !out.assemble_only;
    out
}

fn flag(out: &mut ParsedArgs, name: &str, kind: FlagKind, value: Option<String>) {
    out.flags.push(Flag { kind, name: name.to_string(), value });
}

fn classify(name: &str) -> FlagKind {
    match name {
        "E" | "S" => FlagKind::PreprocessOnly,
        "c" => FlagKind::CompileOnly,
        "s" => FlagKind::AssembleOnly,
        "o" => FlagKind::Output,
        _ => FlagKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> String { s.to_string() }

    #[test]
    fn parses_common_invocations() {
        let p = parse_args(&[v("-O2"), v("-g"), v("main.c"), v("-o"), v("a.out")]);
        assert_eq!(p.inputs, vec!["main.c"]);
        assert_eq!(p.output.as_deref(), Some("a.out"));
        assert_eq!(p.opt_level, 2);
        assert!(p.debug);
        assert!(p.link);
    }

    #[test]
    fn stage_flags_disable_link() {
        for f in ["-c", "-E", "-s"] {
            let p = parse_args(&[v(f), v("x.c")]);
            assert!(!p.link, "{f} should not link");
        }
    }

    #[test]
    fn unknown_flags_recorded_not_fatal() {
        let p = parse_args(&[v("--bogus-flag"), v("x.c")]);
        assert!(p.flags.iter().any(|f| f.name == "bogus-flag"));
        assert_eq!(p.inputs, vec!["x.c"]);
    }
}
