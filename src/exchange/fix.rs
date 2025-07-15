use log::{debug, error};
use quickfix::{
    // Config,
    FieldMap, Message, SessionSettings, SocketInitiator,
    // fix_codegen::field::*
};
use std::collections::HashMap;

// minimal example (requires a FIX engine library like quickfix-rs)
// note: Requires setting up a FIX session (configuration, etc.)

pub fn parse_fix_message(message: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    debug!("Parsing FIX message: {}", message);
    // replace this with your FIX parsing logic (e.g., using quickfix-rs)
    // example: Tag=Value|Tag=Value|...
    let mut fields: HashMap<String, String> = HashMap::new();
    for part in message.split("|") {
        if let Some(eq_pos) = part.find("=") {
            let tag = &part[..eq_pos];
            let value = &part[eq_pos + 1..];
            fields.insert(tag.to_string(), value.to_string());
        }
    }
    Ok(fields)
}

pub fn format_fix_message(fields: &HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Formatting FIX message");
    // replace this with your FIX formatting logic (e.g., using quickfix-rs)
    let mut message = String::new();
    for (tag, value) in fields {
        message.push_str(&format!("{}={}|", tag, value));
    }
    Ok(message)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_message_formatting_parsing() {
        let mut fields = HashMap::new();
        fields.insert("8".to_string(), "FIX.4.2".to_string());
        fields.insert("35".to_string(), "A".to_string());
        fields.insert("49".to_string(), "SenderCompID".to_string());
        fields.insert("56".to_string(), "TargetCompID".to_string());

        let fix_message = format_fix_message(&fields).unwrap();
        let parsed_fields = parse_fix_message(&fix_message).unwrap();

        assert_eq!(fields.len(), parsed_fields.len());
        for (tag, value) in &fields {
            assert_eq!(parsed_fields.get(tag).unwrap(), value);
        }
    }
}
