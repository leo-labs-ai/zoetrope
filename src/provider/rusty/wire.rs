use serde_json::Value;

pub fn parse_line(line: &str) -> Option<Value> {
    serde_json::from_str(line).ok().filter(Value::is_object)
}

pub fn is_header(v: &Value) -> bool {
    v["kind"] == "session.created" && v["ts_ms"].is_number() && v["data"].is_object()
}
