use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GoofyToml {
    pub(crate) package: Package,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
}
