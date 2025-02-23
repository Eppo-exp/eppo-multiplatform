use std::borrow::Cow;

use chrono::{DateTime, Utc};

use crate::{
    bandits::{BanditConfiguration, BanditResponse},
    ufc::UniversalFlagConfig,
    Str,
};

/// Remote configuration for the eppo client. It's a central piece that defines client behavior.
#[derive(Debug)]
pub struct Configuration {
    /// Timestamp when configuration was fetched by the SDK.
    pub(crate) fetched_at: DateTime<Utc>,
    /// Flags configuration.
    pub(crate) flags: UniversalFlagConfig,
    /// Bandits configuration.
    pub(crate) bandits: Option<BanditResponse>,
}

impl Configuration {
    /// Create a new configuration from server responses.
    pub fn from_server_response(
        config: UniversalFlagConfig,
        bandits: Option<BanditResponse>,
    ) -> Configuration {
        let now = Utc::now();

        Configuration {
            fetched_at: now,
            flags: config,
            bandits,
        }
    }

    /// Return a bandit variant for the specified flag key and string flag variation.
    pub(crate) fn get_bandit_key<'a>(&'a self, flag_key: &str, variation: &str) -> Option<&'a Str> {
        self.flags
            .compiled
            .flag_to_bandit_associations
            .get(flag_key)
            .and_then(|x| x.get(variation))
            .map(|variation| &variation.key)
    }

    /// Return bandit configuration for the given key.
    ///
    /// Returns `None` if bandits are missing for bandit does not exist.
    pub(crate) fn get_bandit(&self, bandit_key: &str) -> Option<&BanditConfiguration> {
        self.bandits.as_ref()?.bandits.get(bandit_key)
    }

    /// Returns an iterator over all flag keys. Note that this may return both disabled flags and
    /// flags with bad configuration. Mostly useful for debugging.
    pub fn flag_keys(&self) -> impl Iterator<Item = &Str> {
        self.flags.compiled.flags.keys()
    }

    /// Returns an iterator over all bandit keys. Mostly useful to debugging.
    pub fn bandit_keys(&self) -> impl Iterator<Item = &Str> {
        self.bandits.iter().flat_map(|it| it.bandits.keys())
    }

    /// Returns bytes representing flags configuration.
    ///
    /// The return value should be treated as opaque and passed on to another Eppo client for
    /// initialization.
    pub fn get_flags_configuration(&self) -> Option<Cow<[u8]>> {
        Some(Cow::Borrowed(self.flags.to_json()))
    }

    /// Returns bytes representing bandits configuration.
    ///
    /// The return value should be treated as opaque and passed on to another Eppo client for
    /// initialization.
    pub fn get_bandits_configuration(&self) -> Option<Cow<[u8]>> {
        let bandits = self.bandits.as_ref()?;
        serde_json::to_vec(bandits)
            .inspect_err(|err| {
                log::warn!(target: "eppo", "failed to serialize bandits: {err:?}");
            })
            .ok()
            .map(Cow::Owned)
    }
}
