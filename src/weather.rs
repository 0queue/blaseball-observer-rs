fn weather(w: u64) -> String {
    log::info!("weather: {}", w);
    match w {
        0 => "Void",
        1 => "Sun 2", // formerly Sunny
        2 => "Overcast",
        3 => "Rainy",
        4 => "Sandstorm",
        5 => "Snowy",
        6 => "Acidic",
        7 => "Solar Eclipse",
        8 => "Glitter",
        9 => "Blooddrain",
        10 => "Peanuts",
        11 => "Lots of Birds",
        12 => "Feedback",
        13 => "Reverb",
        14 => "Black Hole",
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

