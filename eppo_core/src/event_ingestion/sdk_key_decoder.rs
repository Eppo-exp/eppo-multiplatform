use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::prelude::*;

const PATH: &str = "v0/i";

pub fn decode_event_ingestion_url(sdk_key: &str) -> Option<String> {
    let encoded_payload = sdk_key.split('.').nth(1)?;
    let decoded_bytes = URL_SAFE_NO_PAD.decode(encoded_payload).ok()?;
    let decoded_str = String::from_utf8(decoded_bytes).ok()?;
    let mut params = form_urlencoded::parse(decoded_str.as_bytes());
    let hostname = params
        .find_map(|(key, value)| if key == "eh" { Some(value) } else { None })?;
    let host_and_path = if hostname.ends_with('/') {
        format!("{}{}", hostname, PATH)
    } else {
        format!("{}/{}", hostname, PATH)
    };

    if !host_and_path.starts_with("http://") && !host_and_path.starts_with("https://") {
        Some(format!("https://{}", host_and_path))
    } else {
        Some(host_and_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    #[test]
    fn decode_succeeds_with_valid_sdk_key() {
        let sdk_key = "zCsQuoHJxVPp895.ZWg9MTIzNDU2LmUudGVzdGluZy5lcHBvLmNsb3Vk";
        let hostname = decode_event_ingestion_url(sdk_key);
        assert_eq!(
            Some("https://123456.e.testing.eppo.cloud/v0/i".to_string()),
            hostname
        );
    }

    #[test]
    fn decode_with_non_url_safe_characters() {
        let invalid_url = "eh=12+3456/.e.testing.eppo.cloud";
        let encoded_payload = URL_SAFE_NO_PAD.encode(invalid_url);
        let sdk_key = format!("zCsQuoHJxVPp895.{encoded_payload}");
        let hostname = decode_event_ingestion_url(&sdk_key);
        assert_eq!(
            Some("https://12 3456/.e.testing.eppo.cloud/v0/i".to_string()),
            hostname
        );
    }

    #[test]
    fn returns_none_when_no_event_hostname() {
        let no_payload = "zCsQuoHJxVPp895";
        let invalid_payload = "zCsQuoHJxVPp895.xxxxxx";

        assert_eq!(None, decode_event_ingestion_url(no_payload));
        assert_eq!(None, decode_event_ingestion_url(invalid_payload));
    }
}
