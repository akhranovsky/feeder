use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;

#[derive(Debug, Deserialize)]
pub struct PlayParams {
    pub url: url::Url,
    #[serde(deserialize_with = "deserialize_action")]
    pub action: Option<PlayAction>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlayAction {
    Passthrough,
    Silence,
    Lang(String),
}

static SUPPORTED_LANGUAGES: [&str; 3] = ["en", "nl", "fr"];

fn is_lang_supported(lang: &str) -> bool {
    SUPPORTED_LANGUAGES.iter().any(|&v| v == lang)
}

fn deserialize_action<'de, D>(de: D) -> Result<Option<PlayAction>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(value) = Option::<String>::deserialize(de)? else {
        return Ok(Some(PlayAction::Passthrough))
    };

    match value.to_lowercase().as_str() {
        "passthrough" => Ok(Some(PlayAction::Passthrough)),
        "silence" => Ok(Some(PlayAction::Silence)),
        lang if is_lang_supported(lang) => Ok(Some(PlayAction::Lang(lang.into()))),
        value => Err(Error::custom(format!(
            "expected Passthrough, Silence or Lang({SUPPORTED_LANGUAGES:?}), received {value}"
        ))),
    }
}
