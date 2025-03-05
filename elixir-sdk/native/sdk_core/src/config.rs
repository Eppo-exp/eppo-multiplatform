use rustler::NifStruct;

#[derive(NifStruct)]
#[module = "EppoSdk.Core.Config"]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub is_graceful_mode: bool,
    pub poll_interval_seconds: Option<u64>,
    pub poll_jitter_seconds: u64,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Invalid value for api_key: cannot be blank".to_string());
        }
        Ok(())
    }
}