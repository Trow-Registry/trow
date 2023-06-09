use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::StorageDriverError;

/*
There are implementation details in this interface that could/should be abstracted out.
*/

mod history_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f %Z";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    pub digest: String,
    #[serde(with = "history_date_format")]
    pub date: DateTime<Utc>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ManifestHistory {
    #[serde(rename = "image")]
    tag: String,
    history: Vec<HistoryEntry>,
}

impl ManifestHistory {
    pub fn new(tag: String) -> ManifestHistory {
        ManifestHistory {
            tag,
            history: Vec::new(),
        }
    }

    pub fn insert(&mut self, digest: String, date: DateTime<Utc>) {
        self.history.push(HistoryEntry { digest, date });
    }

    /*
    Normally deserialized rather than returned like this.
    */
    pub fn _catalog(&self) -> &Vec<HistoryEntry> {
        &self.history
    }
}

#[axum::async_trait]
pub trait CatalogOperations {
    /// Returns a vec of all repository names in the registry
    /// Can optionally be given a start value and maximum number of results to return.
    async fn get_catalog(
        &self,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError>;

    /// Returns a vec of all tags under the given repository
    /// Start value and num_results used to control number of returned results
    /// Allows for some optimisations.
    async fn get_tags(
        &self,
        repo: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError>;

    /// Returns the history for a given tag (what digests it has pointed to)
    async fn get_history(
        &self,
        repo: &str,
        name: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<ManifestHistory, StorageDriverError>;
}
