use regex::Regex;

use crate::domain::release::Version;

pub fn parse_tag(tag_name: &str) -> Option<(String, Version)> {
    // Kotlin regex: ^(.*)-v(\d+).(\d+).(\d+)(-RC\d+)?$
    // Note: the dots are unescaped there, so they match any char; we intentionally mirror that.
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
