#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub localized_name: String,
}

impl Module {
    pub fn as_csv_row(&self) -> String {
        format!("{}, {}", self.name, self.localized_name)
    }

    pub fn as_html_row(&self) -> String {
        format!(
            "<tr>\n  <td>{name}</td>\n  <td>{localized}</td>\n</tr>",
            name = self.name,
            localized = self.localized_name
        )
    }
}
