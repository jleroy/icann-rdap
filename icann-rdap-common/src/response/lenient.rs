//! Types for more lenient processing of invalid RDAP

use std::{fmt::Display, marker::PhantomData, str::FromStr};

use {
    serde::{de::Visitor, Deserialize, Deserializer, Serialize},
    serde_json::Number,
};

use crate::check::StringListCheck;

/// A type that is suppose to be a vector of strings.
///
/// Provides a choice between a string or a vector of strings for deserialization.
///
/// This type is provided to be lenient with misbehaving RDAP servers that
/// serve a string when they are suppose to be serving an array of
/// strings.
///
/// Use one of the From methods for construction.
/// ```rust
/// use icann_rdap_common::prelude::*;
///
/// let v = VectorStringish::from(vec!["one".to_string(), "two".to_string()]);
///
/// // or
///
/// let v = VectorStringish::from("one".to_string());
/// ````
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct VectorStringish {
    vec: Vec<String>,
    #[serde(skip)]
    is_string: bool,
}

impl<'de> Deserialize<'de> for VectorStringish {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(VectorStringishVisitor)
    }
}

struct VectorStringishVisitor;

impl<'de> Visitor<'de> for VectorStringishVisitor {
    type Value = VectorStringish;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expected an array of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(VectorStringish {
            vec: vec![v.to_owned()],
            is_string: true,
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut v = vec![];
        loop {
            let n = seq.next_element()?;
            if let Some(s) = n {
                v.push(s);
            } else {
                break;
            }
        }
        Ok(VectorStringish {
            vec: v,
            is_string: false,
        })
    }
}

impl From<String> for VectorStringish {
    fn from(value: String) -> Self {
        VectorStringish {
            vec: vec![value],
            is_string: false,
        }
    }
}

impl From<&str> for VectorStringish {
    fn from(value: &str) -> Self {
        VectorStringish {
            vec: vec![value.to_owned()],
            is_string: false,
        }
    }
}

impl From<Vec<String>> for VectorStringish {
    fn from(value: Vec<String>) -> Self {
        VectorStringish {
            vec: value,
            is_string: false,
        }
    }
}

impl From<VectorStringish> for Vec<String> {
    fn from(value: VectorStringish) -> Self {
        value.vec
    }
}

impl From<&VectorStringish> for Vec<String> {
    fn from(value: &VectorStringish) -> Self {
        value.vec.to_owned()
    }
}

impl VectorStringish {
    /// Consumes and converts it to a `Vec<String>`.
    pub fn into_vec(self) -> Vec<String> {
        self.vec
    }

    /// Gets a reference to the underlying `Vec<String>`.
    pub fn vec(&self) -> &Vec<String> {
        &self.vec
    }

    /// Returns true if the deserialization was as a string.
    pub fn is_string(&self) -> bool {
        self.is_string
    }
}

impl StringListCheck for VectorStringish {
    fn is_empty_or_any_empty_or_whitespace(&self) -> bool {
        self.vec().is_empty_or_any_empty_or_whitespace()
    }

    fn is_ldh_string_list(&self) -> bool {
        self.vec().is_ldh_string_list()
    }
}

/// Returns `Some(VectorStringish)` if the vector is not empty, otherwise `None`.
pub fn to_opt_vectorstringish(vec: Vec<String>) -> Option<VectorStringish> {
    (!vec.is_empty()).then_some(VectorStringish::from(vec))
}

pub(crate) static EMPTY_VEC_STRING: Vec<String> = vec![];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
enum BoolishInner {
    /// Valid RDAP.
    Bool(bool),

    /// Invalide RDAP.
    String(String),
}

/// A type that is suppose to be a boolean.
///
/// Provides a choice between a boolean or a string representation of a boolean for deserialization.
///
/// This type is provided to be lenient with misbehaving RDAP servers that
/// serve a string representation of a boolean when they are suppose to be serving a boolean
///
/// Use one of the From methods for construction.
/// ```rust
/// use icann_rdap_common::prelude::*;
///
/// let v = Boolish::from(true);
/// ````
///
/// When converting from a string (as would happen with deserialization),
/// the values "true", "t", "yes", and "y" (case-insensitive with whitespace trimmed)
/// will be true, all other values will be false.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct Boolish {
    inner: BoolishInner,
}

impl From<bool> for Boolish {
    fn from(value: bool) -> Self {
        Boolish {
            inner: BoolishInner::Bool(value),
        }
    }
}

impl Display for Boolish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_bool())
    }
}

impl Boolish {
    /// Converts to a bool.
    pub fn into_bool(&self) -> bool {
        match &self.inner {
            BoolishInner::Bool(value) => *value,
            BoolishInner::String(value) => Boolish::is_true(value),
        }
    }

    /// Returns true if the deserialization was as a string.
    pub fn is_string(&self) -> bool {
        match &self.inner {
            BoolishInner::Bool(_) => false,
            BoolishInner::String(_) => true,
        }
    }

    fn is_true(value: &str) -> bool {
        let s = value.trim().to_lowercase();
        s == "true" || s == "t" || s == "yes" || s == "y"
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
enum NumberishInner {
    /// Valid RDAP.
    Number(Number),

    /// Invalide RDAP.
    String(String),
}

/// A type that is suppose to be a number.
///
/// Provides a choice between a number or a string representation of a number for deserialization.
///
/// This type is provided to be lenient with misbehaving RDAP servers that
/// serve a string representation of a number when they are suppose to be serving a number.
///
/// Use the From methods for construction.
/// ```rust
/// use icann_rdap_common::prelude::*;
///
/// let v = Numberish::from(123);
/// ````
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct Numberish<T> {
    inner: NumberishInner,
    phatom: PhantomData<T>,
}

impl<T> From<T> for Numberish<T>
where
    Number: From<T>,
{
    fn from(value: T) -> Self {
        Numberish {
            inner: NumberishInner::Number(Number::from(value)),
            phatom: PhantomData,
        }
    }
}

impl<T> Display for Numberish<T>
where
    Number: From<T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.as_u64()
                .map_or("RANGE_ERRROR".to_string(), |u| u.to_string())
        )
    }
}

impl<T> Numberish<T>
where
    Number: From<T>,
{
    /// Returns true if the deserialization was as a string.
    pub fn is_string(&self) -> bool {
        match &self.inner {
            NumberishInner::Number(_) => false,
            NumberishInner::String(_) => true,
        }
    }

    /// If the `Number` is an integer, represent it as u64 if possible. Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match &self.inner {
            NumberishInner::Number(n) => n.as_u64(),
            NumberishInner::String(s) => Number::from_str(s).ok()?.as_u64(),
        }
    }

    /// If the `Number` is an integer, represent it as u32 if possible. Returns None otherwise.
    pub fn as_u32(&self) -> Option<u32> {
        match &self.inner {
            NumberishInner::Number(n) => n.as_u64()?.try_into().ok(),
            NumberishInner::String(s) => Number::from_str(s).ok()?.as_u64()?.try_into().ok(),
        }
    }

    /// If the `Number` is an integer, represent it as u16 if possible. Returns None otherwise.
    pub fn as_u16(&self) -> Option<u16> {
        match &self.inner {
            NumberishInner::Number(n) => n.as_u64()?.try_into().ok(),
            NumberishInner::String(s) => Number::from_str(s).ok()?.as_u64()?.try_into().ok(),
        }
    }

    /// If the `Number` is an integer, represent it as u8 if possible. Returns None otherwise.
    pub fn as_u8(&self) -> Option<u8> {
        match &self.inner {
            NumberishInner::Number(n) => n.as_u64()?.try_into().ok(),
            NumberishInner::String(s) => Number::from_str(s).ok()?.as_u64()?.try_into().ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        serde_json::{from_str, to_string},
    };

    //
    // VectorStringish tests
    //

    #[test]
    fn test_vectorstringish_serialize_many() {
        // GIVEN
        let many = VectorStringish::from(vec!["one".to_string(), "two".to_string()]);

        // WHEN
        let serialized = to_string(&many).unwrap();

        // THEN
        assert_eq!(serialized, r#"["one","two"]"#);
    }

    #[test]
    fn test_vectorstringish_serialize_one() {
        // GIVEN
        let one = VectorStringish::from("one".to_string());

        // WHEN
        let serialized = to_string(&one).unwrap();

        // THEN
        assert_eq!(serialized, r#"["one"]"#);
    }

    #[test]
    fn test_vectorstringish_deserialize_many() {
        // GIVEN
        let json_str = r#"["one","two"]"#;

        // WHEN
        let deserialized: VectorStringish = from_str(json_str).unwrap();

        // THEN
        assert_eq!(
            deserialized.vec(),
            &vec!["one".to_string(), "two".to_string()]
        );

        // and THEN is not string
        assert!(!deserialized.is_string())
    }

    #[test]
    fn test_vectorstringish_deserialize_one() {
        // GIVEN
        let json_str = r#""one""#;

        // WHEN
        let deserialized: VectorStringish = from_str(json_str).unwrap();

        // THEN
        assert_eq!(deserialized.vec(), &vec!["one".to_string()]);

        // and THEN is string
        assert!(deserialized.is_string())
    }

    //
    // Boolish tests
    //

    #[test]
    fn test_boolish_serialize_bool() {
        // GIVEN
        let b = Boolish::from(true);

        // WHEN
        let serialized = to_string(&b).unwrap();

        // THEN
        assert_eq!(serialized, "true");
    }

    #[test]
    fn test_boolish_deserialize_bool_true() {
        // GIVEN
        let json_str = "true";

        // WHEN
        let deserialized: Boolish = from_str(json_str).unwrap();

        // THEN
        assert!(deserialized.into_bool());
        assert!(!deserialized.is_string());
    }

    #[test]
    fn test_boolish_deserialize_bool_false() {
        // GIVEN
        let json_str = "false";

        // WHEN
        let deserialized: Boolish = from_str(json_str).unwrap();

        // THEN
        assert!(!deserialized.into_bool());
        assert!(!deserialized.is_string());
    }

    #[test]
    fn test_boolish_deserialize_string_true() {
        // GIVEN
        let json_str = r#""true""#;

        // WHEN
        let deserialized: Boolish = from_str(json_str).unwrap();

        // THEN
        assert!(deserialized.into_bool());
        assert!(deserialized.is_string());
    }

    #[test]
    fn test_boolish_deserialize_string_false() {
        // GIVEN
        let json_str = r#""false""#;

        // WHEN
        let deserialized: Boolish = from_str(json_str).unwrap();

        // THEN
        assert!(!deserialized.into_bool());
        assert!(deserialized.is_string());
    }

    #[test]
    fn test_boolish_is_true() {
        // GIVEN various true values
        let true_values = ["true", "t", "yes", "y", " True ", " T ", " Yes ", " Y "];

        // THEN all are true
        for value in true_values {
            assert!(Boolish::is_true(value));
        }
    }

    #[test]
    fn test_boolish_is_false() {
        // GIVEN various false values
        let false_values = ["false", "f", "no", "n", "False", "blah", "1", "0", ""];

        // THEN all are false
        for value in false_values {
            assert!(!Boolish::is_true(value));
        }
    }

    #[test]
    fn test_boolish_from_bool() {
        assert!(Boolish::from(true).into_bool());
        assert!(!Boolish::from(false).into_bool());
    }

    //
    // Numberish Tests
    //

    #[test]
    fn test_numberish_serialize_number() {
        // GIVEN a Numberish from a number
        let n = Numberish::<u32>::from(123);

        // WHEN serialized
        let serialized = to_string(&n).unwrap();

        // THEN it is the correct string
        assert_eq!(serialized, "123");
    }

    #[test]
    fn test_numberish_deserialize_number() {
        // GIVEN a JSON string representing a number
        let json_str = "123";

        // WHEN deserialized
        let deserialized: Numberish<u32> = from_str(json_str).unwrap();

        // THEN the value is correct and it's not a string
        assert_eq!(deserialized.as_u32(), Some(123));
        assert!(!deserialized.is_string());
    }

    #[test]
    fn test_numberish_deserialize_string() {
        // GIVEN a JSON string representing a number as a string
        let json_str = r#""123""#;

        // WHEN deserialized
        let deserialized: Numberish<u32> = from_str(json_str).unwrap();

        // THEN the value is correct and it's a string
        assert_eq!(deserialized.as_u32(), Some(123));
        assert!(deserialized.is_string());
    }

    #[test]
    fn test_numberish_as_u64_number() {
        // GIVEN a Numberish from a u64
        let n = Numberish::from(123u64);

        // WHEN as_u64 is called
        let result = n.as_u64();

        // THEN the result is Some(123)
        assert_eq!(result, Some(123));
    }

    #[test]
    fn test_numberish_as_u64_string_invalid() {
        // GIVEN a Numberish from a string that does not represent a u64
        let n = Numberish {
            inner: NumberishInner::String("abc".to_string()),
            phatom: PhantomData::<u64>,
        };

        // WHEN as_u64 is called
        let result = n.as_u64();

        // THEN the result is None
        assert_eq!(result, None);
    }

    #[test]
    fn test_numberish_as_smaller_types() {
        // GIVEN a valid number
        let n = Numberish::from(123u64);

        // THEN smaller type conversions work
        assert_eq!(n.as_u32(), Some(123));
        assert_eq!(n.as_u16(), Some(123));
        assert_eq!(n.as_u8(), Some(123));

        // GIVEN a number too large
        let n = Numberish::from(u32::MAX as u64 + 1);

        // THEN smaller type conversions fail
        assert_eq!(n.as_u32(), None);
        assert_eq!(n.as_u16(), None);
        assert_eq!(n.as_u8(), None);

        // GIVEN a valid number string
        let n = Numberish {
            inner: NumberishInner::String("123".to_string()),
            phatom: PhantomData::<u64>,
        };

        // THEN smaller type conversions work
        assert_eq!(n.as_u32(), Some(123));
        assert_eq!(n.as_u16(), Some(123));
        assert_eq!(n.as_u8(), Some(123));

        // GIVEN a number string too large
        let n = Numberish {
            inner: NumberishInner::String((u32::MAX as u64 + 1).to_string()),
            phatom: PhantomData::<u64>,
        };

        // THEN smaller type conversions fail
        assert_eq!(n.as_u32(), None);
    }

    #[test]
    fn test_numberish_display_number() {
        let n = Numberish::<u32>::from(123);
        assert_eq!(format!("{}", n), "123");
    }

    #[test]
    fn test_numberish_display_string_valid() {
        let n = Numberish {
            inner: NumberishInner::String("123".to_string()),
            phatom: PhantomData::<u32>,
        };
        assert_eq!(format!("{}", n), "123");
    }

    #[test]
    fn test_numberish_display_string_invalid() {
        let n = Numberish {
            inner: NumberishInner::String("abc".to_string()),
            phatom: PhantomData::<u32>,
        };
        assert_eq!(format!("{}", n), "RANGE_ERRROR");
    }
}
