use core::fmt;
use std::collections::HashMap;

use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor},
};

use crate::maven_central::pom::pom::Properties;

impl<'de> Deserialize<'de> for Properties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PropertiesVisitor;

        impl<'de> Visitor<'de> for PropertiesVisitor {
            type Value = Properties;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of XML properties")
            }

            // quick-xml represents child elements as a map during deserialization
            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut map = HashMap::new();

                // Iterate through every child tag
                while let Some(key) = access.next_key::<String>()? {
                    // Force the value to be parsed as a map that contains a "$value" (text) field
                    // This bypasses the "expected string, found map" error
                    #[derive(Deserialize)]
                    struct RawValue {
                        #[serde(rename = "$value", default)]
                        content: String,
                    }

                    let value: RawValue = access.next_value()?;
                    map.insert(key, value.content);
                }

                Ok(Properties { map })
            }
        }

        deserializer.deserialize_map(PropertiesVisitor)
    }
}
