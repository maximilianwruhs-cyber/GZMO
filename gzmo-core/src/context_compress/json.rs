use serde_json::Value;

pub fn compress_json(content: &str, row_cap: usize) -> Result<String, serde_json::Error> {
    let val: Value = serde_json::from_str(content)?;
    let crushed = crush_value(val, row_cap, 0);
    serde_json::to_string(&crushed)
}

fn crush_value(val: Value, row_cap: usize, depth: usize) -> Value {
    if depth > 3 {
        return val;
    }
    match val {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Value::Array(arr);
            }
            
            let is_array_of_objects = arr.iter().all(|v| v.is_object());
            let original_len = arr.len();

            let mut crushed_arr = Vec::with_capacity(std::cmp::min(original_len, row_cap + 1));
            
            for item in arr.into_iter().take(row_cap) {
                crushed_arr.push(crush_value(item, row_cap, depth + 1));
            }

            if original_len > row_cap {
                let keys: Vec<String> = if is_array_of_objects {
                    if let Some(Value::Object(map)) = crushed_arr.first() {
                        map.keys().cloned().collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                
                let mut summary = serde_json::Map::new();
                summary.insert("_omitted_rows".to_string(), Value::from(original_len - row_cap));
                if !keys.is_empty() {
                    summary.insert("_keys".to_string(), Value::from(keys));
                }
                crushed_arr.push(Value::Object(summary));
            }

            Value::Array(crushed_arr)
        }
        Value::Object(map) => {
            let mut crushed_map = serde_json::Map::new();
            for (k, v) in map.into_iter() {
                crushed_map.insert(k, crush_value(v, row_cap, depth + 1));
            }
            Value::Object(crushed_map)
        }
        Value::String(s) => {
            if s.len() > 1000 {
                Value::String(format!(
                    "[string of length {}; prefix: {}]",
                    s.len(),
                    &s[..std::cmp::min(s.len(), 60)]
                ))
            } else {
                Value::String(s)
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_json_array() {
        let mut arr = Vec::new();
        for i in 0..50 {
            let mut obj = serde_json::Map::new();
            obj.insert("id".to_string(), Value::from(i));
            obj.insert("name".to_string(), Value::from(format!("item-{}", i)));
            arr.push(Value::Object(obj));
        }
        let json_str = serde_json::to_string(&Value::Array(arr)).unwrap();
        let compressed = compress_json(&json_str, 5).unwrap();
        let parsed: Value = serde_json::from_str(&compressed).unwrap();
        
        let crushed_arr = parsed.as_array().unwrap();
        assert_eq!(crushed_arr.len(), 6); // 5 elements + 1 summary element
        
        let summary = crushed_arr.last().unwrap().as_object().unwrap();
        assert_eq!(summary.get("_omitted_rows").unwrap().as_i64().unwrap(), 45);
        let keys: Vec<&str> = summary.get("_keys").unwrap().as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        assert!(keys.contains(&"id"));
        assert!(keys.contains(&"name"));
    }

    #[test]
    fn test_compress_json_long_string() {
        let mut obj = serde_json::Map::new();
        let long_str = "A".repeat(1500);
        obj.insert("description".to_string(), Value::from(long_str));
        let json_str = serde_json::to_string(&Value::Object(obj)).unwrap();
        let compressed = compress_json(&json_str, 5).unwrap();
        assert!(compressed.contains("[string of length 1500; prefix:"));
    }
}
