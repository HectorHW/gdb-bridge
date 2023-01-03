use lazy_static::lazy_static;

use regex::Regex;

use serde::Deserialize;

use serde_aux::prelude::*;

use super::de_octal::deserialize_number_from_octal_str;

fn transform_to_json(input: &str) -> String {
    assert!(input.starts_with("*stopped"));
    //as we are provided with almost dict-like structure, we could try just parsing it as json
    let mut result = String::new();
    result.push('{');

    let input = input.strip_prefix("*stopped,").unwrap();

    lazy_static! {
        static ref KEY_WITH_EQ: Regex = Regex::new(r"([a-z\-]+)=").unwrap();
    }

    //fix keys

    let fixed_keys = KEY_WITH_EQ.replace_all(input, "\"$1\":");

    result.push_str(&fixed_keys);

    result.push('}');

    result
}

use serde_json::{Result as JResult, Value};

pub fn jsonize_stop_message(message: &str) -> JResult<Value> {
    let enclosed = transform_to_json(message);

    serde_json::from_str(&enclosed)
}

pub fn parse_stop_message(message: &str) -> JResult<StopReason> {
    let enclosed = transform_to_json(message);

    serde_json::from_str(&enclosed)
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SignalReceived {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub core: usize,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub thread_id: usize,
    pub signal_name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ExitedWithCode {
    #[serde(deserialize_with = "deserialize_number_from_octal_str")]
    pub exit_code: usize,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "reason")]
#[serde(rename_all = "kebab-case")]
pub enum StopReason {
    ExitedNormally,
    SignalReceived(SignalReceived),
    Exited(ExitedWithCode),
}
