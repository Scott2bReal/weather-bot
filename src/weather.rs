use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Location {
    key: String,
    localized_name: String,
    country: Country,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.localized_name, self.country.id)
    }
}

#[derive(Deserialize, Debug)]
pub struct Country {
    #[serde(alias = "ID")]
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Forecast {
    pub headline: Headline,
}

#[derive(Deserialize, Debug)]
pub struct Headline {
    #[serde(alias = "Text")]
    pub overview: String,
}
