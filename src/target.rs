//! Compilation target descriptions (gcc `-march`/`--target` equivalents).

/// ABI family the generated code targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abi {
    /// System V AMD64 ABI.
    SysV,
    /// AAPCS64.
    AAPCS64,
    /// SigmaOS bare-metal kernel ABI.
    SigmaKernel,
}

/// A concrete compilation target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub arch: String,
    pub os: String,
    pub env: String,
    pub abi: Abi,
}

impl Target {
    /// Parse a `arch-vendor-os-env` style triple (vendor ignored, like gcc).
    pub fn parse(triple: &str) -> Option<Self> {
        let parts: Vec<&str> = triple.split('-').collect();
        if parts.len() < 3 {
            return None;
        }
        let (arch, os) = (parts[0].to_string(), parts[2].to_string());
        let env = parts.get(3).unwrap_or(&"").to_string();
        let abi = match arch.as_str() {
            "x86_64" | "amd64" => Abi::SysV,
            "aarch64" | "arm64" => Abi::AAPCS64,
            "sigma" | "sigmaos" => Abi::SigmaKernel,
            _ => return None,
        };
        Some(Self { arch, os, env, abi })
    }

    /// Host default target.
    pub fn host() -> Self {
        Self::parse("x86_64-unknown-linux-gnu").unwrap()
    }
}

/// Resolved tool/spec set for a target — what gcc calls a "spec".
#[derive(Debug, Clone)]
pub struct TargetSpec {
    pub target: Target,
    pub pic: bool,
    pub pie: bool,
    pub debug: bool,
    pub opt_level: u8,
}

impl TargetSpec {
    /// Build the spec a driver would select for `target` + flag profile.
    pub fn for_target(target: Target, pic: bool, pie: bool, debug: bool, opt_level: u8) -> Self {
        Self { target, pic, pie, debug, opt_level }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_triples() {
        let t = Target::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(t.arch, "x86_64");
        assert_eq!(t.abi, Abi::SysV);
        assert_eq!(Target::parse("aarch64-linux-android").unwrap().abi, Abi::AAPCS64);
        assert_eq!(Target::parse("sigma-bare-none").unwrap().abi, Abi::SigmaKernel);
        assert!(Target::parse("mips-linux").is_none());
        assert!(Target::parse("x86_64").is_none());
    }
}
