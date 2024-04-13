#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GoofyToml {
    pub(crate) package: Package,
    pub(crate) dependencies: Option<HashMap<String, DependencyInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum DependencyInfo {
    simple(String),
    details(DependencyDetails),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DependencyDetails {
    pub(crate) version: String,
}
