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

            if !v.is_ascii() {
                // TODO this is a workaround for the new Tokyo Lift,
                //   who are represented by the 🏋️‍♀️ emoji, which is
                //   combining.  Proper emoji handling should probably now be string-wise
                //   Should probably also add a switch to turn off combining emojis, as
                //   the terminals I use don't seem to like them very much
                return v.chars().next().ok_or_else(|| { E::custom("No chars found in string") })
            }

            u32::from_str_radix(v.trim_start_matches("0x"), 16)
                .map_err(E::custom)
                .and_then(|u| {
                    std::char::from_u32(u).ok_or_else(|| E::custom("not an emoji"))
                })
        }
    }

    deserializer.deserialize_any(EmojiVisitor)
}

// NOTE: seems to work better than {:2}, but in some terminals (or fonts maybe?)
//       the width is still inconsistent (like the intellij terminal).
//       Gnome terminal seems good though, so w/e
pub fn pad(emoji: char) -> String {
    match unicode_width::UnicodeWidthChar::width(emoji) {
        Some(2) => emoji.to_string(),
        _ => format!("{} ", emoji),
    }
}

