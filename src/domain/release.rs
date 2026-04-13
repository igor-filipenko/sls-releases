use std::cmp::Ordering;

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
    pub version: Version,
    pub url: String,
    pub date_time: String,
}

impl Release {
    pub fn as_csv_row(&self) -> String {
        format!(
            "{}, {}, {}, {}",
            self.name, self.localized_name, self.version, self.url
        )
    }

    pub fn as_html_row(&self, base_url: &str, use_candidate: bool) -> String {
        format!(
            "<tr>\n  <td><a href='{base_url}/{name}?rc={use_candidate}'>{name}</a></td>\n  <td>{localized}</td>\n  <td><a href='{url}'>{version}</a></td>\n</tr>",
            name = self.name,
            localized = self.localized_name,
            url = self.url,
            version = self.version
        )
    }
}

impl Ord for Release {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
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

impl ModuleRelease {
    pub fn as_csv_row(&self) -> String {
        format!("{}, {}, {}", self.version, self.date_time, self.url)
    }

    pub fn as_html_row(&self) -> String {
        format!(
            "<tr>\n  <td><a href='{url}'>{version}</a></td>\n  <td>{date_time}</td>\n</tr>",
            url = self.url,
            version = self.version,
            date_time = self.date_time
        )
    }
}
