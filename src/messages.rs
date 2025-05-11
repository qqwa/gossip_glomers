use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Body {
    Init {
        msg_id: u64,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn test_init_msg() {
        let msg = r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#;
        let got: Message = serde_json::from_str(&msg).unwrap();
        let want = Message {
            src: "c0".to_string(),
            dest: "n0".to_string(),
            body: super::Body::Init {
                msg_id: 1,
                node_id: "n0".to_string(),
                node_ids: vec!["n0".to_string()],
            },
        };
        assert_eq!(got, want);
    }
}
