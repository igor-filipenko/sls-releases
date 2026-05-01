use serde::{Deserialize, Serialize};

use crate::domain::module::Module as DomainModule;

#[derive(Debug, Deserialize)]
pub struct ModulesQuery {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Module {
    pub name: String,
    pub localized_name: String,
}

impl From<DomainModule> for Module {
    fn from(value: DomainModule) -> Self {
        Self {
            name: value.name,
            localized_name: value.localized_name,
        }
    }
}
