use super::Gatherer;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RegistryErrors {
    #[error("gatherer `{0}` not found")]
    GathererNotFoundError(String),
    #[error("could not extract the gatherer version from {0}, version should follow <gathererName>@<version> syntax")]
    GathererNameAndVersionError(String),
}

pub struct GatherersRegistry {
    gatherers: HashMap<String, HashMap<String, Arc<dyn Gatherer>>>,
}

impl GatherersRegistry {
    pub fn get_gatherer(self, name: String) -> Result<Arc<dyn Gatherer>, RegistryErrors> {
        let (gatherer_name, version) = extract_version_and_gatherer_name(&name)?;

        let latest_version =
            version.unwrap_or(self.get_latest_version_for_gatherer(&gatherer_name)?);

        match self
            .gatherers
            .get(&gatherer_name)
            .and_then(|versioned_gatherers| versioned_gatherers.get(&latest_version))
        {
            Some(gatherer) => Ok(gatherer.clone()),
            None => Err(RegistryErrors::GathererNotFoundError(name)),
        }
    }

    pub fn inspect_gatherers(self) -> Vec<String> {
        let mut gatherers_list: Vec<String> = vec![];
        for (gatherer_name, versions) in self.gatherers {
            let mut sorted_versions: Vec<String> = versions.keys().cloned().collect();
            sorted_versions.sort();

            gatherers_list.push(format!("{} - {}", gatherer_name, sorted_versions.join("/")));
        }

        gatherers_list
    }

    fn get_latest_version_for_gatherer(&self, name: &str) -> Result<String, RegistryErrors> {
        match self.gatherers.get(name) {
            Some(versioned_gatherers) => {
                let mut versions: Vec<String> = versioned_gatherers.keys().cloned().collect();
                versions.sort();
                Ok(versions.last().unwrap().to_owned())
            }
            None => Err(RegistryErrors::GathererNotFoundError(name.to_owned())),
        }
    }
}

fn extract_version_and_gatherer_name(
    gatherer_name: &str,
) -> Result<(String, Option<String>), RegistryErrors> {
    let parts: Vec<&str> = gatherer_name.split("@").collect();

    if parts.len() == 1 {
        return Ok((parts[0].to_owned(), None));
    }
    if parts.len() != 2 {
        return Err(RegistryErrors::GathererNameAndVersionError(
            gatherer_name.to_owned(),
        ));
    }
    Ok((parts[0].to_owned(), Some(parts[1].to_owned())))
}

pub struct GatherersRegistryBuilder {
    gatherers: Vec<(String, String, Arc<dyn Gatherer>)>,
}

impl GatherersRegistryBuilder {
    pub fn new() -> GatherersRegistryBuilder {
        GatherersRegistryBuilder {
            gatherers: Vec::new(),
        }
    }

    pub fn add_gatherer(
        &mut self,
        name: &str,
        version: &str,
        gatherer: impl Gatherer + 'static,
    ) -> &mut GatherersRegistryBuilder {
        self.gatherers
            .push((name.to_owned(), version.to_owned(), Arc::new(gatherer)));

        self
    }

    pub fn build_registry(self) -> GatherersRegistry {
        let mut gatherers_map: HashMap<String, HashMap<String, Arc<dyn Gatherer>>> = HashMap::new();

        for (name, version, gatherer) in self.gatherers {
            match gatherers_map.get_mut(&name) {
                Some(versioned_gatherers) => {
                    versioned_gatherers.insert(version, gatherer);
                }
                None => {
                    let mut versioned_gatherer = HashMap::new();
                    versioned_gatherer.insert(version, gatherer);
                    gatherers_map.insert(name, versioned_gatherer);
                }
            };
        }

        GatherersRegistry {
            gatherers: gatherers_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gatherers::MockGatherer;

    #[test]
    fn test_registry_building() {
        let mockgatherer = MockGatherer::new();
        let mockgatherer_another = MockGatherer::new();
        let mockgatherer_other = MockGatherer::new();

        let mut builder = GatherersRegistryBuilder::new();
        builder.add_gatherer("test_gatherer", "v1", mockgatherer);
        builder.add_gatherer("test_gatherer", "v2", mockgatherer_another);
        builder.add_gatherer("another_test", "v1", mockgatherer_other);

        let registry = builder.build_registry();

        let available = registry.inspect_gatherers();

        assert!(available.contains(&"another_test - v1".to_owned()));
        assert!(available.contains(&"test_gatherer - v1/v2".to_owned()));
    }

    #[test]
    fn test_registry_get_gatherer_invalid_name_format() {
        let mockgatherer = MockGatherer::new();
        let mut builder = GatherersRegistryBuilder::new();
        builder.add_gatherer("test_gatherer", "v1", mockgatherer);
        let registry = builder.build_registry();

        let registry_error = registry
            .get_gatherer("other@v2@v2".to_owned())
            .err()
            .unwrap();

        assert_eq!(
            registry_error,
            RegistryErrors::GathererNameAndVersionError("other@v2@v2".to_owned())
        )
    }

    #[test]
    fn test_registry_get_gatherer_not_found() {
        let mockgatherer = MockGatherer::new();
        let mut builder = GatherersRegistryBuilder::new();
        builder.add_gatherer("test_gatherer", "v1", mockgatherer);
        let registry = builder.build_registry();

        let registry_error = registry.get_gatherer("other".to_owned()).err().unwrap();

        assert_eq!(
            registry_error,
            RegistryErrors::GathererNotFoundError("other".to_owned())
        )
    }

    #[test]
    fn test_registry_get_gatherer_not_found_with_version() {
        let mockgatherer = MockGatherer::new();
        let mut builder = GatherersRegistryBuilder::new();
        builder.add_gatherer("test_gatherer", "v1", mockgatherer);
        let registry = builder.build_registry();

        let registry_error = registry.get_gatherer("other@v1".to_owned()).err().unwrap();

        assert_eq!(
            registry_error,
            RegistryErrors::GathererNotFoundError("other@v1".to_owned())
        )
    }

    #[test]
    fn test_registry_get_gather_found_with_version() {
        let mut mockgatherer = MockGatherer::new();

        mockgatherer
            .expect_name()
            .with()
            .times(1)
            .returning(|| "test_gatherer".to_owned());

        let mut builder = GatherersRegistryBuilder::new();
        builder.add_gatherer("test_gatherer", "v1", mockgatherer);
        builder.add_gatherer("test_gatherer", "v2", MockGatherer::new());

        let registry = builder.build_registry();

        let gatherer = registry
            .get_gatherer("test_gatherer@v1".to_owned())
            .unwrap();

        assert_eq!(gatherer.name(), "test_gatherer".to_owned())
    }

    #[test]
    fn test_registry_get_gather_found_without_version() {
        let mut mockgatherer = MockGatherer::new();

        mockgatherer
            .expect_name()
            .with()
            .times(1)
            .returning(|| "test_gatherer_v2".to_owned());

        let mut builder = GatherersRegistryBuilder::new();
        builder.add_gatherer("test_gatherer", "v1", MockGatherer::new());
        builder.add_gatherer("test_gatherer", "v2", mockgatherer);

        let registry = builder.build_registry();

        let gatherer = registry.get_gatherer("test_gatherer".to_owned()).unwrap();

        assert_eq!(gatherer.name(), "test_gatherer_v2".to_owned())
    }
}
