use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize, Deserialize)]
pub(crate) struct  GoofyToml{
    pub(crate) package:Package,
}


#[derive(Debug,Serialize, Deserialize)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[cfg(test)]
fn test_package_toml() {
    let Goofy_toml = "[package]
        name = \"Qmm - learning\"
        version = \"0.0.1\"";

    let package = toml::from_str(Goofy_toml).unwrap();
}
