use super::StorageDriverError;
use crate::types::ManifestHistory;
pub trait CatalogOperations {
    /// Returns a vec of all repository names in the registry
    /// Can optionally be given a start value and maximum number of results to return.
    fn get_catalog(
        &self,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError>;

    /// Returns a vec of all tags under the given repository
    /// Start value and num_results used to control number of returned results
    /// Allows for some optimisations.
    fn get_tags(
        &self,
        repo: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError>;

    /// Returns the history for a given tag (what digests it has pointed to)
    fn get_history(
        &self,
        repo: &str,
        name: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<ManifestHistory, StorageDriverError>;
}