use regex::Regex;
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

impl std::fmt::Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) {}",
            self.name, self.localized_name, self.version
        )
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

pub fn parse_tag(tag_name: &str) -> Option<(String, Version)> {
    let re = Regex::new(r"^(.*)-v(\d+).(\d+).(\d+)(-RC\d+)?$").ok()?;
    let caps = re.captures(tag_name)?;

    let module = caps.get(1)?.as_str().to_string();
    let major: i32 = caps.get(2)?.as_str().parse().ok()?;
    let minor: i32 = caps.get(3)?.as_str().parse().ok()?;
    let patch: i32 = caps.get(4)?.as_str().parse().ok()?;

    let version = match caps.get(5).map(|m| m.as_str()) {
        Some(suffix) => {
            let number: i32 = suffix.strip_prefix("-RC")?.parse().ok()?;
            Version::Candidate {
                major,
                minor,
                patch,
                number,
            }
        }
        None => Version::Release {
            major,
            minor,
            patch,
        },
    };

    Some((module, version))
}
