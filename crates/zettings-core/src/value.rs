#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum SettingValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    TextList(Vec<String>),
}

impl SettingValue {
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Text(_) => "text",
            Self::TextList(_) => "text-list",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_tagged() {
        let json = serde_json::to_string(&SettingValue::Bool(true)).unwrap();
        assert_eq!(json, r#"{"type":"bool","value":true}"#);
        let back: SettingValue = serde_json::from_str(&json).unwrap();
        assert_eq!(back, SettingValue::Bool(true));
    }

    #[test]
    fn type_names_are_stable() {
        assert_eq!(SettingValue::TextList(vec![]).type_name(), "text-list");
        assert_eq!(SettingValue::Int(-3).type_name(), "int");
    }
}
