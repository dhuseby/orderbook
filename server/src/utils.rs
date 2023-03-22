use oberon::PublicKey;
use serde::{
    de::{self, SeqAccess, Unexpected, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serializer,
};
use std::fmt::{self, Formatter};
use url::Url;
use uuid::Uuid;

/// Method for serializing Uuids
pub fn serialize_uuid<S: Serializer>(i: &Uuid, s: S) -> std::result::Result<S::Ok, S::Error> {
    if s.is_human_readable() {
        s.serialize_str(&i.to_string())
    } else {
        let bytes = i.as_bytes();
        let mut tupler = s.serialize_tuple(bytes.len())?;
        for b in bytes {
            tupler.serialize_element(b)?;
        }
        tupler.end()
    }
}

/// Method for deserializing Uuids
pub fn deserialize_uuid<'de, D>(d: D) -> std::result::Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    struct UuidVisitor;

    impl<'de> Visitor<'de> for UuidVisitor {
        type Value = Uuid;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            write!(formatter, "a string")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            match Uuid::parse_str(v) {
                Err(_) => Err(de::Error::invalid_value(Unexpected::Str(v), &self)),
                Ok(i) => Ok(i),
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut bytes = [0u8; 16];
            let mut i = 0;
            while let Some(b) = seq.next_element()? {
                bytes[i] = b;
                i += 1;
            }
            Ok(Uuid::from_bytes(bytes))
        }
    }

    if d.is_human_readable() {
        d.deserialize_str(UuidVisitor)
    } else {
        d.deserialize_tuple(16, UuidVisitor)
    }
}

/// Method for deserializing a Url
pub fn url_de<'de, D>(deserializer: D) -> std::result::Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    // get the string representation...
    let s = String::deserialize(deserializer)?;
    // parse it into a Url
    Url::parse(&s).map_err(serde::de::Error::custom)
}

/// Method for deserializing an Option<Url>
pub fn url_de_opt<'de, D>(deserializer: D) -> std::result::Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    // get the string representation...
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    // parse it into a Url
    match Url::parse(&s) {
        Ok(url) => Ok(Some(url)),
        Err(e) => Err(serde::de::Error::custom(e)),
    }
}

/// Method for deserializing an Oberon PublicKey
pub fn pk_de<'de, D>(deserializer: D) -> std::result::Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    // get the string representation...
    let mut s = String::deserialize(deserializer)?;
    // remove everything that isn't hex
    s.retain(|c| c.is_ascii_hexdigit());
    // decode it to a slice
    let mut bytes = [0u8; 288];
    hex::decode_to_slice(s.as_str(), &mut bytes as &mut [u8])
        .map_err(|_| serde::de::Error::custom("failed to decode hex"))?;
    // convert it to a PublicKey
    let pk = PublicKey::from_bytes(&bytes);
    if pk.is_some().unwrap_u8() == 1u8 {
        Ok(pk.unwrap())
    } else {
        Err(serde::de::Error::custom("failed to deserialize PublicKey"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;

    #[derive(Deserialize)]
    struct PkTest {
        #[serde(deserialize_with = "pk_de")]
        public_key: PublicKey,
    }

    #[test]
    fn pk() {
        let pk_str = r#"
            public_key = '''
            80506cb367e4fd5d537b30b5feae71d4414db2d1ba7de1b6bfdaf0eeb0efacabe72cb843
            8c9929f94713378b726703860b4faf6bb4314df131a2722e65cf0dea251396802c89d6ef
            c2a86fa65de362e9c998261ccf4b27db6930ca079178981bacdb6b8602d36bf8b0745023
            c3c87cfdc91e54ecb7b31ad02f720a9e7facfcfb898a30728b45f2708dd07bc0b5f97155
            1143e62867c05c4ac85efd7c09262b11c43c934db6730afc50e1360803a6df57aa1c7d3f
            4618f5e9be38ca4b606e0fff88b6e8fbfed09826f62faf87a4ef432370137b5b015acda3
            ca516a8a46ff99106e6773bdecd0137bf367ee4cb520c64112e3c82f01beefe4d7cf78f5
            78c44de1ea6915ae0eb7643fb92f5287d57b625730bb2e5f5abe9ad112baa478be2af9c4
            '''
        "#;

        let _pk: PkTest = toml::from_str(pk_str).unwrap();
    }

    #[derive(Deserialize)]
    struct UrlTest {
        #[serde(deserialize_with = "url_de")]
        pub url: Url,
    }

    #[test]
    fn url() {
        let url_str = r#"
            url = "https://foo.com:8080/blah"
        "#;

        let _url: UrlTest = toml::from_str(url_str).unwrap();
    }

    #[derive(Deserialize)]
    struct OptUrlTest {
        #[serde(deserialize_with = "url_de_opt")]
        pub url: Option<Url>,
    }

    #[test]
    fn opt_url_some() {
        let url_str_some = r#"
            url = "https://foo.com:8080/blah"
        "#;

        let url: OptUrlTest = toml::from_str(url_str_some).unwrap();
        assert!(url.url.is_some());
    }

    #[test]
    fn opt_url_none() {
        let url_str_none = r#"
            url = ""
        "#;

        let url: OptUrlTest = toml::from_str(url_str_none).unwrap();
        assert!(url.url.is_none());
    }
}
