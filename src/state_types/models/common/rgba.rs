use serde::de::Unexpected;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RGBA {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl RGBA {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        RGBA {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn transparent() -> Self {
        RGBA::new(0, 0, 0, 0)
    }
}

impl Serialize for RGBA {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.red, self.green, self.blue, self.alpha
        )
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RGBA {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Deserialize::deserialize(deserializer) as Result<String, D::Error> {
            Ok(ref hex_color) => {
                if hex_color.len() != 9 {
                    Err(de::Error::invalid_length(hex_color.len(), &"9"))
                } else if !hex_color.starts_with(&"#") {
                    Err(de::Error::custom(format!(
                        "invalid color {}, must starts with #",
                        hex_color
                    )))
                } else {
                    let red = u8::from_str_radix(&hex_color[1..3], 16);
                    let green = u8::from_str_radix(&hex_color[3..5], 16);
                    let blue = u8::from_str_radix(&hex_color[5..7], 16);
                    let alpha = u8::from_str_radix(&hex_color[7..9], 16);
                    match (red, green, blue, alpha) {
                        (Ok(red), Ok(green), Ok(blue), Ok(alpha)) => Ok(RGBA {
                            red,
                            green,
                            blue,
                            alpha,
                        }),
                        _ => Err(de::Error::custom(format!(
                            "invalid color {}, all values must be of type u8",
                            hex_color
                        ))),
                    }
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RGBA;

    #[test]
    fn serde_rgba() {
        let color = RGBA {
            red: 10,
            green: 20,
            blue: 30,
            alpha: 40,
        };
        let ser = serde_json::to_string(&color).unwrap();
        let deser = serde_json::from_str(&ser).unwrap();
        assert_eq!(color, deser, "color serialized and deserialized correctly");
    }
}
