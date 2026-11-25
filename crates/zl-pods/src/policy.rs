//! Installation and update policy evaluation (`docs/spec/pods.md` §6).

use std::fmt;

use crate::version::PodVersion;

/// Evaluation outcome when comparing candidate package against installed status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyOutcome {
    /// Pod ID is not currently installed.
    New,
    /// Pod is installed with same signer and candidate version is higher.
    Update,
    /// Pod is installed with same signer and identical version.
    SameVersion,
    /// Pod is installed with same signer but candidate version is lower.
    Downgrade,
    /// Candidate package is signed with a different key than established install.
    SignerChanged,
}

impl PolicyOutcome {
    /// Canonical string identifier defined in specification.
    pub const fn as_str(self) -> &'static str {
        match self {
            PolicyOutcome::New => "NEW",
            PolicyOutcome::Update => "UPDATE",
            PolicyOutcome::SameVersion => "SAME_VERSION",
            PolicyOutcome::Downgrade => "DOWNGRADE",
            PolicyOutcome::SignerChanged => "SIGNER_CHANGED",
        }
    }
}

impl fmt::Display for PolicyOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Evaluates installation policy for a candidate version and signer.
pub fn evaluate_policy(
    installed: Option<(&PodVersion, &str)>,
    candidate_version: &PodVersion,
    candidate_signer: &str,
) -> PolicyOutcome {
    let (inst_version, inst_signer) = match installed {
        Some((v, s)) => (v, s),
        None => return PolicyOutcome::New,
    };

    if inst_signer != candidate_signer {
        return PolicyOutcome::SignerChanged;
    }

    match candidate_version.cmp(inst_version) {
        std::cmp::Ordering::Greater => PolicyOutcome::Update,
        std::cmp::Ordering::Equal => PolicyOutcome::SameVersion,
        std::cmp::Ordering::Less => PolicyOutcome::Downgrade,
    }
}
