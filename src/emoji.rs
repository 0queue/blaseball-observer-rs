pub fn deserialize<'de, D>(deserializer: D) -> Result<char, D::Error> where D: serde::Deserializer<'de> {
    struct EmojiVisitor;

    impl<'de> serde::de::Visitor<'de> for EmojiVisitor {
        type Value = char;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing an emoji")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
        {
            u32::from_str_radix(v.trim_start_matches("0x"), 16)
                .map_err(E::custom)
                .and_then(|u| {
                    std::char::from_u32(u).ok_or_else(|| E::custom("not an emoji"))
                })
        }
    }

    deserializer.deserialize_str(EmojiVisitor)
}

