use url::Url;

use crate::Str;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SdkKey(Str);

/// Blank implementation of `Debug`, so we don't accidentally leak SDK key in logs.
impl std::fmt::Debug for SdkKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "SdkKey{..}")
    }
}

impl SdkKey {
    #[inline]
    pub fn new(key: Str) -> SdkKey {
        SdkKey(key)
    }

    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Decodes and returns the event ingestion URL from the provided Eppo SDK key.
    ///
    /// If the SDK key doesn't contain the event ingestion hostname, or it's invalid, it returns
    /// `None`.
    pub(crate) fn event_ingestion_url(&self) -> Option<Url> {
        use base64::prelude::*;

        let (_, payload) = self.as_str().split_once(".")?;
        let payload = BASE64_URL_SAFE_NO_PAD.decode(payload).ok()?;
        let (_, hostname) =
            url::form_urlencoded::parse(&payload).find(|(key, _value)| key == "eh")?;

        let s = {
            let mut s = String::new();
            if !hostname.starts_with("http://") && !hostname.starts_with("https://") {
                // Prefix with a scheme if missing.
                s.push_str("https://");
            }
            s.push_str(&hostname);
            if !s.ends_with('/') {
                s.push('/');
            }
            s.push_str("v0/i");
            s
        };

        let url = Url::parse(&s).ok()?;

        Some(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_none_when_no_ingestion_url() {
        assert_eq!(
            SdkKey::new("zCsQuoHJxVPp895".into()).event_ingestion_url(),
            None
        );
        assert_eq!(
            SdkKey::new("zCsQuoHJxVPp895.xxxxxx".into()).event_ingestion_url(),
            None
        );
    }

    #[test]
    fn test_decode_event_ingestion_url() {
        assert_eq!(
            SdkKey::new("zCsQuoHJxVPp895.ZWg9MTIzNDU2LmUudGVzdGluZy5lcHBvLmNsb3Vk".into())
                .event_ingestion_url(),
            Some(Url::parse("https://123456.e.testing.eppo.cloud/v0/i").unwrap())
        )
    }

    #[test]
    fn test_decode_non_url_safe_event_ingestion_url() {
        use base64::prelude::*;

        let payload = BASE64_URL_SAFE_NO_PAD.encode("eh=12%3D3456(lol)%2Bhi/.e+testing.eppo.cloud");
        let sdk_key = SdkKey::new(format!("zCsQuoHJxVPp895.{payload}").into());

        assert_eq!(
            sdk_key.event_ingestion_url(),
            // Believe it or not, that's a valid url syntax
            Some(Url::parse("https://12=3456(lol)+hi/.e testing.eppo.cloud/v0/i").unwrap())
        )
    }
}
