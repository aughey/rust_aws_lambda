use serde::Serialize;

/// This is a made-up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
pub struct Response {
    pub req_id: String,
    pub instances: Vec<Instance>,
}

#[derive(Serialize)]
pub struct Instance {
    pub id: String,
    pub state: String,
    pub ip: Option<String>
}
