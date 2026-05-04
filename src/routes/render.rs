use crate::domain::module::Module;
use crate::domain::release::{ModuleRelease, Release};

pub fn release_csv_row(release: &Release) -> String {
    format!(
        "{}, {}, {}, {}",
        release.name, release.localized_name, release.version, release.url
    )
}

pub fn release_html_row(
    release: &Release,
    base_url: &str,
    use_candidate: bool,
    use_milestones: bool,
) -> String {
    format!(
        "<tr>\n  <td><a href='{base_url}/{name}?rc={use_candidate}&ms={use_milestones}'>{name}</a></td>\n  <td>{localized}</td>\n  <td><a href='{url}'>{version}</a></td>\n</tr>",
        name = release.name,
        localized = release.localized_name,
        url = release.url,
        version = release.version
    )
}

pub fn module_release_csv_row(r: &ModuleRelease) -> String {
    format!("{}, {}, {}", r.version, r.date_time, r.url)
}

pub fn module_release_html_row(r: &ModuleRelease) -> String {
    format!(
        "<tr>\n  <td><a href='{url}'>{version}</a></td>\n  <td>{date_time}</td>\n</tr>",
        url = r.url,
        version = r.version,
        date_time = r.date_time
    )
}

pub fn module_csv_row(m: &Module) -> String {
    format!("{}, {}", m.name, m.localized_name)
}

pub fn module_html_row(m: &Module) -> String {
    format!(
        "<tr>\n  <td>{name}</td>\n  <td>{localized}</td>\n</tr>",
        name = m.name,
        localized = m.localized_name
    )
}

pub fn releases_table_html(
    base_url: &str,
    use_candidate: bool,
    use_milestones: bool,
    releases: &[Release],
) -> String {
    let rows = releases
        .iter()
        .map(|r| release_html_row(r, base_url, use_candidate, use_milestones))
        .collect::<Vec<_>>()
        .join("\n");
    format!("<table rules=\"all\">{rows}</table>")
}

pub fn module_releases_table_html(module_releases: &[ModuleRelease]) -> String {
    let rows = module_releases
        .iter()
        .map(module_release_html_row)
        .collect::<Vec<_>>()
        .join("\n");
    format!("<table rules=\"all\">{rows}</table>")
}

pub fn releases_csv(releases: &[Release]) -> String {
    releases
        .iter()
        .map(release_csv_row)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

pub fn module_releases_csv(module_releases: &[ModuleRelease]) -> String {
    module_releases
        .iter()
        .map(module_release_csv_row)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

pub fn modules_table_html(modules: &[Module]) -> String {
    let rows = modules
        .iter()
        .map(module_html_row)
        .collect::<Vec<_>>()
        .join("\n");
    format!("<table rules=\"all\">{rows}</table>")
}

pub fn modules_csv(modules: &[Module]) -> String {
    modules
        .iter()
        .map(module_csv_row)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
