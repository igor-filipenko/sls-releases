use crate::domain::release::{ModuleRelease, Release};

pub fn releases_table_html(
    base_url: &str,
    use_candidate: bool,
    use_milestones: bool,
    releases: &[Release],
) -> String {
    let rows = releases
        .iter()
        .map(|r| r.as_html_row(base_url, use_candidate, use_milestones))
        .collect::<Vec<_>>()
        .join("\n");
    format!("<table rules=\"all\">{rows}</table>")
}

pub fn module_releases_table_html(module_releases: &[ModuleRelease]) -> String {
    let rows = module_releases
        .iter()
        .map(|r| r.as_html_row())
        .collect::<Vec<_>>()
        .join("\n");
    format!("<table rules=\"all\">{rows}</table>")
}

pub fn releases_csv(releases: &[Release]) -> String {
    releases
        .iter()
        .map(|r| r.as_csv_row())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

pub fn module_releases_csv(module_releases: &[ModuleRelease]) -> String {
    module_releases
        .iter()
        .map(|r| r.as_csv_row())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
