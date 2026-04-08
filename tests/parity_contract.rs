use sls_releases::clients::github::parse::parse_tag;
use sls_releases::domain::release::{ModuleRelease, Release, Version};
use sls_releases::render;

fn kotlin_to_boolean(s: Option<&str>) -> bool {
    // Kotlin String?.toBoolean(): returns true iff equals("true", ignoreCase=true)
    // In routes it is: parameters.get("rc").toBoolean()
    // parameters.get("rc") can be null -> false
    matches!(s, Some(v) if v.eq_ignore_ascii_case("true"))
}

#[test]
fn rc_query_param_semantics_match_kotlin() {
    assert!(!kotlin_to_boolean(None));
    assert!(kotlin_to_boolean(Some("true")));
    assert!(kotlin_to_boolean(Some("TRUE")));
    assert!(!kotlin_to_boolean(Some("false")));
    assert!(!kotlin_to_boolean(Some("1")));
    assert!(!kotlin_to_boolean(Some("yes")));
}

#[test]
fn version_ordering_matches_kotlin_release_gt_candidate() {
    let rc1 = Version::Candidate {
        major: 1,
        minor: 2,
        patch: 3,
        number: 1,
    };
    let rc2 = Version::Candidate {
        major: 1,
        minor: 2,
        patch: 3,
        number: 2,
    };
    let rel = Version::Release {
        major: 1,
        minor: 2,
        patch: 3,
    };

    assert!(rc1 < rc2);
    assert!(rc2 < rel);
    assert_eq!(rel.to_string(), "1.2.3");
    assert_eq!(rc2.to_string(), "1.2.3-RC2");
}

#[test]
fn parse_tag_recognizes_release_and_candidate() {
    let (m1, v1) = parse_tag("accumulations-v1.2.3").unwrap();
    assert_eq!(m1, "accumulations");
    assert_eq!(
        v1,
        Version::Release {
            major: 1,
            minor: 2,
            patch: 3
        }
    );

    let (m2, v2) = parse_tag("accumulations-v1.2.3-RC7").unwrap();
    assert_eq!(m2, "accumulations");
    assert_eq!(
        v2,
        Version::Candidate {
            major: 1,
            minor: 2,
            patch: 3,
            number: 7
        }
    );
}

#[test]
fn release_html_row_matches_kotlin_shape() {
    let r = Release {
        name: "accumulations".into(),
        localized_name: "Накопления".into(),
        version: Version::Release {
            major: 1,
            minor: 2,
            patch: 3,
        },
        url: "https://example/release".into(),
        date_time: "ignored".into(),
    };

    let html = r.as_html_row("/sls/releases", false);
    assert!(html.contains("<tr>"));
    assert!(html.contains("href='/sls/releases/accumulations?rc=false'"));
    assert!(html.contains(">Накопления</td>"));
    assert!(html.contains("href='https://example/release'"));
    assert!(html.contains(">1.2.3</a>"));
}

#[test]
fn html_table_wrapper_matches_kotlin_prefix_postfix() {
    let r = Release {
        name: "m".into(),
        localized_name: "m".into(),
        version: Version::Release {
            major: 1,
            minor: 0,
            patch: 0,
        },
        url: "u".into(),
        date_time: "d".into(),
    };

    let table = render::releases_table_html("/sls/releases", true, &[r]);
    assert!(table.starts_with("<table rules=\"all\">"));
    assert!(table.ends_with("</table>"));
}

#[test]
fn csv_has_trailing_newline_like_kotlin() {
    let r = Release {
        name: "a".into(),
        localized_name: "b".into(),
        version: Version::Release {
            major: 1,
            minor: 2,
            patch: 3,
        },
        url: "u".into(),
        date_time: "d".into(),
    };

    let csv = render::releases_csv(&[r]);
    assert!(csv.ends_with('\n'));
    assert!(csv.contains("a, b, 1.2.3, u"));
}

#[test]
fn module_release_rows_match_kotlin_shapes() {
    let mr = ModuleRelease {
        version: Version::Candidate {
            major: 1,
            minor: 0,
            patch: 0,
            number: 9,
        },
        url: "https://example/mr".into(),
        date_time: "Jan 1, 2026 at 1:23 PM".into(),
    };

    let html = mr.as_html_row();
    assert!(html.contains("href='https://example/mr'"));
    assert!(html.contains(">1.0.0-RC9</a>"));
    assert!(html.contains(">Jan 1, 2026 at 1:23 PM</td>"));

    let csv = mr.as_csv_row();
    assert_eq!(csv, "1.0.0-RC9, Jan 1, 2026 at 1:23 PM, https://example/mr");
}
