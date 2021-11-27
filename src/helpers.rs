use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// Copied from cargo/crates_io
mod crates_io {
    #[derive(super::Deserialize)]
    pub struct PublishedCrateDependency {
        pub optional: bool,
        pub default_features: bool,
        pub name: String,
        pub features: Vec<String>,
        pub version_req: String,
        #[serde(default)]
        pub target: Option<String>,
        pub kind: String,
        #[serde(default)]
        pub registry: Option<String>,
        #[serde(default)]
        pub explicit_name_in_toml: Option<String>,
    }

    #[derive(super::Deserialize)]
    pub struct PublishedCrate {
        pub name: String,
        pub vers: semver::Version,
        pub deps: Vec<PublishedCrateDependency>,
        pub features: super::BTreeMap<String, Vec<String>>,
        pub authors: Vec<String>,
        pub description: Option<String>,
        pub documentation: Option<String>,
        pub homepage: Option<String>,
        pub readme: Option<String>,
        pub readme_file: Option<String>,
        pub keywords: Vec<String>,
        pub categories: Vec<String>,
        pub license: Option<String>,
        pub license_file: Option<String>,
        pub repository: Option<String>,
        pub badges: super::BTreeMap<String, super::BTreeMap<String, String>>,
        pub links: Option<String>,
        #[serde(default)]
        pub v: Option<u32>,
    }
}

// Copied from cargo::sources::registry.
mod cargo {

    #[derive(super::Serialize)]
    pub struct RegistryPackage {
        pub name: String,
        pub vers: semver::Version,
        pub deps: Vec<RegistryDependency>,
        pub features: super::BTreeMap<String, Vec<String>>,
        pub cksum: String,
        pub yanked: bool,
        pub links: Option<String>,
        pub v: Option<u32>,
    }

    #[derive(super::Serialize)]
    pub struct RegistryDependency {
        pub name: String,
        pub req: String,
        pub features: Vec<String>,
        pub optional: bool,
        pub default_features: bool,
        pub target: Option<String>,
        pub kind: Option<String>,
        pub registry: Option<String>,
        pub package: Option<String>,
        pub public: Option<bool>,
    }
}

pub use cargo::*;
pub use crates_io::*;

pub fn crate_to_package(input: PublishedCrate, cksum: String) -> RegistryPackage {
    RegistryPackage {
        name: input.name,
        vers: input.vers,
        deps: input
            .deps
            .into_iter()
            .map(|dep| {
                let (name, package) = match &dep.explicit_name_in_toml {
                    Some(explicit) => (explicit.to_string(), Some(dep.name.to_string())),
                    None => (dep.name.to_string(), None),
                };
                RegistryDependency {
                    name,
                    req: dep.version_req,
                    features: dep.features,
                    optional: dep.optional,
                    default_features: dep.default_features,
                    target: dep.target,
                    kind: Some(dep.kind),
                    registry: dep.registry,
                    package,
                    public: Some(true),
                }
            })
            .collect(),
        features: input.features,
        cksum,
        yanked: false,
        links: None,
        v: input.v,
    }
}
