fn weather(w: u64) -> String {
    match w {
        1 => "Sunny",
        7 => "Solar Eclipse",
        9 => "Blooddrain",
        10 => "Peanuts",
        11 => "Lots of Birds",
        12 => "Feedback",
        13 => "Reverb",
        _ => "?",
    }.to_string()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error> where D: serde::Deserializer<'de> {
    struct WeatherVisitor;

    impl<'de> serde::de::Visitor<'de> for WeatherVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number representing the weather")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
        {
            Ok(weather(v))
        }
    }

    deserializer.deserialize_any(WeatherVisitor)
}

