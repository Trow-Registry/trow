use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/*
There are implementation details in this interface that could/should be abstracted out.
*/

mod history_date_format {
    use chrono::{DateTime, Utc};
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
        s.parse().map_err(serde::de::Error::custom)
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
    image: String,
    history: Vec<HistoryEntry>,
}

impl ManifestHistory {
    pub fn new(image: String, history: Vec<HistoryEntry>) -> ManifestHistory {
        ManifestHistory { image, history }
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
