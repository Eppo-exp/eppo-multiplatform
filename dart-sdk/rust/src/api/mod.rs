use flutter_rust_bridge::frb;

pub use eppo_core::{
    eval::BanditResult,
    events::{AssignmentEvent, BanditEvent},
    Str,
};

pub mod attributes;
pub mod client;

#[frb(external)]
impl Str {}

#[frb(rust2dart(dart_type = "String", dart_code = "{}"))]
pub fn encode_str(raw: Str) -> String {
    raw.as_str().into()
}

#[frb(dart2rust(dart_type = "String", dart_code = "{}"))]
pub fn decode_str(raw: String) -> Str {
    raw.into()
}

// #[frb(external)]
// impl AssignmentEvent {}

#[frb(rust2dart(dart_type = "Map<String, dynamic>", dart_code = "json.decode({})"))]
pub fn encode_assignment_event(event: AssignmentEvent) -> String {
    serde_json::to_string(&event).expect("AssignmentEvent should be serializable to JSON")
}

#[frb(dart2rust(dart_type = "Map<String, dynamic>", dart_code = "json.encode({})"))]
pub fn decode_assignment_event(s: String) -> AssignmentEvent {
    // serde_json::from_str(&s).expect("AssignmentEvent should be deserializable from JSON")
    unreachable!("AssignmentEvent is never sent from Dart to Rust");
}

#[frb(rust2dart(dart_type = "Map<String, dynamic>", dart_code = "json.decode({})"))]
pub fn encode_bandit_event(event: BanditEvent) -> String {
    serde_json::to_string(&event).expect("BanditEvent should be serializable to JSON")
}

#[frb(dart2rust(dart_type = "Map<String, dynamic>", dart_code = "json.encode({})"))]
pub fn decode_bandit_event(s: String) -> BanditEvent {
    // serde_json::from_str(&s).expect("BanditEvent should be deserializable from JSON")
    unreachable!("BanditEvent is never sent from Dart to Rust");
}

#[frb(mirror(BanditResult))]
pub struct BanditResultMirror {
    /// Selected variation from the feature flag.
    pub variation: Str,
    /// Selected action if any.
    pub action: Option<Str>,
    /// Flag assignment event that needs to be logged to analytics storage.
    pub assignment_event: Option<AssignmentEvent>,
    /// Bandit assignment event that needs to be logged to analytics storage.
    pub bandit_event: Option<BanditEvent>,
}

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
