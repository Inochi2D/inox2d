//! JSON API wrapper, with methods and structs similar to Java's `org.json`.
//!
//! > I might turn this into my own JSON crate at this point... ¬¬

use glam::{Vec2, Vec3};
use json::JsonValue;

pub(super) trait SerialExtend {
    fn as_object(&self) -> Option<&json::object::Object>;
}

impl SerialExtend for json::JsonValue {
    fn as_object(&self) -> Option<&json::object::Object> {
        if let json::JsonValue::Object(transform) = self {
            Some(transform)
        } else {
            None
        }
    }
}

pub type JsonResult<T> = Result<T, JsonError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum JsonError {
    #[error("Key {0:?} does not exist")]
    KeyDoesNotExist(String),
    #[error("Value at {0:?} is not an object")]
    ValueIsNotObject(String),
    #[error("Value at {0:?} is not a list")]
    ValueIsNotList(String),
    #[error("Value at {0:?} is not a string")]
    ValueIsNotString(String),
    #[error("Value at {0:?} is not a number")]
    ValueIsNotNumber(String),
    #[error("Value at {0:?} is not a bool")]
    ValueIsNotBool(String),
    #[error("Error while parsing int at {0:?}\n  - number out of scope")]
    ParseIntError(String),
    #[error("Error while parsing vec2 at {key:?}\n  - {msg}")]
    ParseVec2Error { key: String, msg: String },
    #[error("Error while parsing vec3 at {key:?}\n  - {msg}")]
    ParseVec3Error { key: String, msg: String },
    #[error("Error in list at index {index}\n  - {inner}")]
    ErrorInList { index: usize, inner: Box<JsonError> },
    #[error("Error in object at {key:?}\n  - {inner}")]
    ErrorInObject { key: String, inner: Box<JsonError> },
}

impl JsonError {
    pub fn nested(self, key: &str) -> Self {
        Self::ErrorInObject {
            key: key.to_owned(),
            inner: Box::new(self),
        }
    }
}

pub struct JsonObject<'a>(pub &'a json::object::Object);

impl<'a> JsonObject<'a> {
    fn get(&self, key: &str) -> JsonResult<&json::JsonValue> {
        match self.0.get(key) {
            Some(value) => Ok(value),
            None => Err(JsonError::KeyDoesNotExist(key.to_owned())),
        }
    }

    pub fn get_object(&self, key: &str) -> JsonResult<JsonObject> {
        match self.get(key)?.as_object() {
            Some(obj) => Ok(JsonObject(obj)),
            None => Err(JsonError::ValueIsNotObject(key.to_owned())),
        }
    }

    pub fn get_list(&self, key: &str) -> JsonResult<&[JsonValue]> {
        match self.get(key)? {
            json::JsonValue::Array(arr) => Ok(arr),
            _ => Err(JsonError::ValueIsNotList(key.to_owned())),
        }
    }

    pub fn get_nullable_str(&self, key: &str) -> JsonResult<Option<&str>> {
        let val = self.get(key)?;
        if val.is_null() {
            return Ok(None);
        }
        match val.as_str() {
            Some(val) => Ok(Some(val)),
            None => Err(JsonError::ValueIsNotString(key.to_owned())),
        }
    }

    pub fn get_str(&self, key: &str) -> JsonResult<&str> {
        match self.get(key)?.as_str() {
            Some(val) => Ok(val),
            None => Err(JsonError::ValueIsNotString(key.to_owned())),
        }
    }

    fn get_number(&self, key: &str) -> JsonResult<json::number::Number> {
        match self.get(key)?.as_number() {
            Some(val) => Ok(val),
            None => Err(JsonError::ValueIsNotNumber(key.to_owned())),
        }
    }

    pub fn get_f64(&self, key: &str) -> JsonResult<f64> {
        Ok(self.get_number(key)?.into())
    }

    pub fn get_f32(&self, key: &str) -> JsonResult<f32> {
        Ok(self.get_number(key)?.into())
    }

    pub fn get_u64(&self, key: &str) -> JsonResult<u64> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_u32(&self, key: &str) -> JsonResult<u32> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_u16(&self, key: &str) -> JsonResult<u16> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_u8(&self, key: &str) -> JsonResult<u8> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_usize(&self, key: &str) -> JsonResult<usize> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_i64(&self, key: &str) -> JsonResult<i64> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_i32(&self, key: &str) -> JsonResult<i32> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_i16(&self, key: &str) -> JsonResult<i16> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_i8(&self, key: &str) -> JsonResult<i8> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_isize(&self, key: &str) -> JsonResult<isize> {
        self.get_number(key)?
            .try_into()
            .map_err(|_| JsonError::ParseIntError(key.to_owned()))
    }

    pub fn get_bool(&self, key: &str) -> JsonResult<bool> {
        match self.get(key)?.as_bool() {
            Some(val) => Ok(val),
            None => Err(JsonError::ValueIsNotBool(key.to_owned())),
        }
    }

    pub fn get_vec2(&self, key: &str) -> JsonResult<Vec2> {
        let list = self.get_list(key)?;
        if list.len() != 2 {
            return Err(JsonError::ParseVec2Error {
                key: key.to_owned(),
                msg: format!("expected list of length 2, but has length {}", list.len()),
            });
        }

        let x = match list[0].as_number() {
            Some(val) => val.into(),
            None => {
                return Err(JsonError::ParseVec2Error {
                    key: key.to_owned(),
                    msg: "expected float, but did not get a number".to_owned(),
                })
            }
        };

        let y = match list[1].as_number() {
            Some(val) => val.into(),
            None => {
                return Err(JsonError::ParseVec2Error {
                    key: key.to_owned(),
                    msg: "expected float, but did not get a number".to_owned(),
                })
            }
        };
        Ok(Vec2::new(x, y))
    }

    pub fn get_vec3(&self, key: &str) -> JsonResult<Vec3> {
        let list = self.get_list(key)?;
        if list.len() != 3 {
            return Err(JsonError::ParseVec3Error {
                key: key.to_owned(),
                msg: format!("expected list of length 3, but has length {}", list.len()),
            });
        }

        let x = match list[0].as_number() {
            Some(val) => val.into(),
            None => {
                return Err(JsonError::ParseVec3Error {
                    key: key.to_owned(),
                    msg: "expected float, but did not get a number".to_owned(),
                })
            }
        };

        let y = match list[1].as_number() {
            Some(val) => val.into(),
            None => {
                return Err(JsonError::ParseVec3Error {
                    key: key.to_owned(),
                    msg: "expected float, but did not get a number".to_owned(),
                })
            }
        };

        let z = match list[2].as_number() {
            Some(val) => val.into(),
            None => {
                return Err(JsonError::ParseVec3Error {
                    key: key.to_owned(),
                    msg: "expected float, but did not get a number".to_owned(),
                })
            }
        };

        Ok(Vec3::new(x, y, z))
    }
}
