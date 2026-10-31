//! Small reusable lexical classification rules shared by all downstream exports.
use serde_json::Value;

pub(crate) fn is_form(senses: &[Value]) -> bool {
    !senses.is_empty()
        && senses.iter().all(|sense| {
            sense
                .get("tags")
                .and_then(Value::as_array)
                .is_some_and(|tags| tags.iter().any(|tag| tag.as_str() == Some("form-of")))
        })
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_pure_inflections_are_hidden_by_default() {
        assert!(is_form(&[json!({"tags":["form-of","plural"]})]));
        assert!(!is_form(&[json!({"tags":[]})]));
        assert!(!is_form(&[json!({"tags":["form-of"]}),json!({"tags":["Mexico"]})]));
        assert!(!is_form(&[]));
    }
}
