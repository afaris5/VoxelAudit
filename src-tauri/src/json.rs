use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::{Map, Number, Value};
use std::{collections::BTreeSet, fmt};

pub struct Document {
    pub value: Value,
    pub duplicates: BTreeSet<String>,
}

impl Document {
    fn scalar(value: Value) -> Self {
        Self {
            value,
            duplicates: BTreeSet::new(),
        }
    }
}

struct DocumentVisitor;

impl<'de> Visitor<'de> for DocumentVisitor {
    type Value = Document;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Document, E> {
        Ok(Document::scalar(Value::Bool(value)))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Document, E> {
        Ok(Document::scalar(Value::Number(value.into())))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Document, E> {
        Ok(Document::scalar(Value::Number(value.into())))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Document, E> {
        Number::from_f64(value)
            .map(|n| Document::scalar(Value::Number(n)))
            .ok_or_else(|| E::custom("number is out of range"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Document, E> {
        Ok(Document::scalar(Value::String(value.into())))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Document, E> {
        Ok(Document::scalar(Value::String(value)))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Document, E> {
        Ok(Document::scalar(Value::Null))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Document, A::Error> {
        let mut values = Vec::new();
        let mut duplicates = BTreeSet::new();
        while let Some(child) = sequence.next_element::<Document>()? {
            for path in child.duplicates {
                if duplicates.len() < 100 {
                    duplicates.insert(format!("/{}{path}", values.len()));
                }
            }
            values.push(child.value);
        }
        Ok(Document {
            value: Value::Array(values),
            duplicates,
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Document, A::Error> {
        let mut values = Map::new();
        let mut duplicates = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            let child = map.next_value::<Document>()?;
            let escaped = key.replace('~', "~0").replace('/', "~1");
            for path in child.duplicates {
                if duplicates.len() < 100 {
                    duplicates.insert(format!("/{escaped}{path}"));
                }
            }
            if values.insert(key, child.value).is_some() && duplicates.len() < 100 {
                duplicates.insert(format!("/{escaped}"));
            }
        }
        Ok(Document {
            value: Value::Object(values),
            duplicates,
        })
    }
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DocumentVisitor)
    }
}

pub fn parse(bytes: &[u8]) -> Result<Document, serde_json::Error> {
    serde_json::from_slice(bytes)
}
