/// Additional payload support for known vendor blocks.
use crate::formats::json::JsonObject;
use crate::formats::payload::{as_nested_list, as_object, InoxParseError, InoxParseResult};
use crate::params::{Binding, ParamUuid};
use json::JsonValue;

pub const SESSION_BINDINGS_KEY: &str = "com.inochi2d.inochi-session.bindings";

pub enum BindingType {
	RatioBinding,
	ExpressionBinding,
}

pub struct SessionBinding<'file> {
	pub name: &'file str,
	pub source_name: &'file str,
	pub source_display_name: &'file str,
	pub source_type: &'file str,
	pub binding_type: BindingType,
	pub param: ParamUuid,
	pub axis: u8,
	pub dampen_level: f32,
}

impl<'file> SessionBinding<'file> {
	pub fn new_from_json_object(object: JsonObject<'file>) -> InoxParseResult<Self> {
		Ok(Self {
			name: object.get_str("name")?,
			source_name: object.get_str("sourceName")?,
			source_display_name: object.get_str("sourceDisplayName")?,
			source_type: object.get_str("sourceType")?,
			binding_type: match object.get_str("bindingType")? {
				"RatioBinding" => BindingType::RatioBinding,
				"ExpressionBinding" => BindingType::ExpressionBinding,
				unknown => return Err(InoxParseError::UnknownVendorKeyValue("bindingType", unknown.to_owned())),
			},
			param: ParamUuid(object.get_u32("param")?),
			axis: object.get_u8("axis")?,
			dampen_level: object.get_f32("dampenLevel")?,
		})
	}

	pub fn new_from_json_list(value: &'file JsonValue) -> InoxParseResult<Vec<Self>> {
		let mut out = vec![];

		for (index, binding) in as_nested_list(0, value)?.iter().enumerate() {
			out.push(Self::new_from_json_object(as_object(&format!("{}", index), binding)?)?);
		}

		Ok(out)
	}
}
