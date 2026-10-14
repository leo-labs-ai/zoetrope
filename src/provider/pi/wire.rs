use serde_json::Value;

pub fn parse_line(line: &str) -> Option<Value> {
    serde_json::from_str(line).ok().filter(Value::is_object)
}

pub fn is_header(v: &Value) -> bool {
    v["type"] == "session" && v["version"] == 3 && v["id"].is_string()
}
