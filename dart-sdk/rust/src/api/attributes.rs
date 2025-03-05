use flutter_rust_bridge::frb;

pub use eppo_core::AttributeValue;

#[frb(external)]
impl AttributeValue {}

#[frb(sync, positional)]
pub fn numeric_attribute(value: f64) -> AttributeValue {
    AttributeValue::numeric(value)
}

#[frb(sync, positional)]
pub fn categorical_number_attribute(value: f64) -> AttributeValue {
    AttributeValue::categorical(value)
}

#[frb(sync, positional)]
pub fn string_attribute(value: String) -> AttributeValue {
    AttributeValue::categorical(value)
}

#[frb(sync, positional)]
pub fn bool_attribute(value: bool) -> AttributeValue {
    AttributeValue::categorical(value)
}
