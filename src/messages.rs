use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl Message {
    pub fn create_response(&self, body: Body) -> Message {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body,
        }
    }
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
    Echo {
        msg_id: u64,
        echo: String,
    },
    EchoOk {
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<u64>,
        in_reply_to: u64,
        echo: String,
    },
    Generate {
        msg_id: u64,
    },
    GenerateOk {
        in_reply_to: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<u64>,
        id: String,
    },
    Broadcast {
        message: serde_json::Value,
        msg_id: u64,
    },
    BroadcastOk {
        in_reply_to: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<u64>,
    },
    Read {
        msg_id: u64,
    },
    ReadOk {
        in_reply_to: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<u64>,
        messages: Vec<serde_json::Value>,
    },
    Topology {
        msg_id: u64,
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {
        in_reply_to: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<u64>,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Body, Message};

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

    #[test]
    fn topology_body() {
        let body = r#"{"type":"topology","topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]},"msg_id":1}"#;
        let got: Body = serde_json::from_str(&body).unwrap();

        let mut topology: HashMap<String, Vec<String>> = HashMap::new();
        topology.insert(
            "n1".to_string(),
            ["n2".to_string(), "n3".to_string()].into(),
        );
        topology.insert("n2".to_string(), ["n1".to_string()].into());
        topology.insert("n3".to_string(), ["n1".to_string()].into());
        let want = Body::Topology {
            msg_id: 1,
            topology,
        };
        assert_eq!(got, want);
    }
}
