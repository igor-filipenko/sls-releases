use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReleaseKind {
    Milestone,
    Production,
    Candidate,
}

impl ReleaseKind {
    fn rank(self) -> i32 {
        match self {
            // Production should win if versions are identical.
            ReleaseKind::Production => 2,
            ReleaseKind::Milestone => 1,
            ReleaseKind::Candidate => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Version {
    Release {
        major: i32,
        minor: i32,
        patch: i32,
    },
    Candidate {
        major: i32,
        minor: i32,
        patch: i32,
        number: i32,
    },
}

impl Version {
    fn parts(&self) -> (i32, i32, i32, i32) {
        match *self {
            Version::Release {
                major,
                minor,
                patch,
            } => (major, minor, patch, i32::MAX),
            Version::Candidate {
                major,
                minor,
                patch,
                number,
            } => (major, minor, patch, number),
        }
    }

    pub fn kind(&self) -> ReleaseKind {
        match self {
            Version::Release { .. } => ReleaseKind::Production,
            Version::Candidate { .. } => ReleaseKind::Candidate,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Version::Release {
                major,
                minor,
                patch,
            } => write!(f, "{major}.{minor}.{patch}"),
            Version::Candidate {
                major,
                minor,
                patch,
                number,
            } => write!(f, "{major}.{minor}.{patch}-RC{number}"),
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.parts().cmp(&other.parts())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Release {
    pub name: String,
    pub localized_name: String,
    pub kind: ReleaseKind,
    pub version: Version,
    pub url: String,
    pub date_time: String,
    /// Only meaningful for `kind == ReleaseKind::Milestone`.
    pub closed: bool,
}

impl Ord for Release {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.version.cmp(&other.version) {
            Ordering::Equal => self.kind.rank().cmp(&other.kind.rank()),
            o => o,
        }
    }
}

impl PartialOrd for Release {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModuleRelease {
    pub version: Version,
    pub url: String,
    pub date_time: String,
}
