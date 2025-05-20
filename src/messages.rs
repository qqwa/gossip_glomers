use std::collections::HashMap;

use serde::{Deserialize, Serialize};
impl Message {
    pub fn create_response(&self, body: Body) -> Message {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Body {
    Init(Init),
    InitOk(InitOk),
    Echo(Echo),
    EchoOk(EchoOk),
    Generate(Generate),
    GenerateOk(GenerateOk),
    Topology(Topology),
    TopologyOk(TopologyOk),
    Broadcast(Broadcast),
    BroadcastOk(BroadcastOk),
    Read(Read),
    ReadOk(ReadOk),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Init {
    pub msg_id: u64,
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct InitOk {
    pub in_reply_to: u64,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Echo {
    pub msg_id: u64,
    pub echo: String,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct EchoOk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    pub in_reply_to: u64,
    pub echo: String,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Generate {
    pub msg_id: u64,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GenerateOk {
    pub in_reply_to: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    pub id: String,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Broadcast {
    pub message: serde_json::Value,
    pub msg_id: u64,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BroadcastOk {
    pub in_reply_to: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Read {
    pub msg_id: u64,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ReadOk {
    pub in_reply_to: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    pub messages: Vec<serde_json::Value>,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Topology {
    pub msg_id: u64,
    pub topology: HashMap<String, Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TopologyOk {
    pub in_reply_to: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
}

macro_rules! impl_from_body {
    ($variant:ident) => {
        impl From<Body> for $variant {
            fn from(body: Body) -> Self {
                if let Body::$variant(inner) = body {
                    inner
                } else {
                    panic!("should not happen");
                }
            }
        }
    };
}

impl_from_body!(Init);
impl_from_body!(InitOk);
impl_from_body!(Echo);
impl_from_body!(EchoOk);
impl_from_body!(Generate);
impl_from_body!(GenerateOk);
impl_from_body!(Broadcast);
impl_from_body!(BroadcastOk);
impl_from_body!(Read);
impl_from_body!(ReadOk);
impl_from_body!(Topology);
impl_from_body!(TopologyOk);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Body, Init, Message, Topology};

    #[test]
    fn test_init_msg() {
        let msg = r#"{"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}"#;
        let got: Message = serde_json::from_str(&msg).unwrap();
        let want = Message {
            src: "c0".to_string(),
            dest: "n0".to_string(),
            body: Body::Init(Init {
                msg_id: 1,
                node_id: "n0".to_string(),
                node_ids: vec!["n0".to_string()],
            }),
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
        let want = Body::Topology(Topology {
            msg_id: 1,
            topology,
        });
        assert_eq!(got, want);
    }
}
