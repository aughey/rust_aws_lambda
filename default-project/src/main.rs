use aws_sdk_ec2::{
    model::{Instance, InstanceStateName},
    output::DescribeInstancesOutput,
    Client,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use serde::{Deserialize};

/// This is a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    command: String,
}

mod Response {
    use serde::Serialize;

    /// This is a made-up example of what a response structure may look like.
    /// There is no restriction on what it can be. The runtime requires responses
    /// to be serialized into json. The runtime pays no attention
    /// to the contents of the response payload.
    #[derive(Serialize,Default)]
    pub struct Response {
        pub req_id: String,
        pub instances: Vec<Instance>,
    }

    #[derive(Serialize,Default)]
    pub struct Instance {
        pub id: String,
        pub state: String,
        pub ip: Option<String>
    }
}

fn instance_reservations<'a>(
    instances: DescribeInstancesOutput,
) -> impl Iterator<Item = Instance> + 'a {
    instances
        .reservations
        .unwrap_or_default()
        .into_iter()
        .filter_map(|reservation| reservation.instances)
        .flatten()
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response::Response, Error> {
    // Extract some useful info from the request
    let command = event.payload.command;

    let config = aws_config::from_env().load().await;

    let client = Client::new(&config);

    let instances = client
        .describe_instances()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let instances = instance_reservations    

    for instance in instance_reservations(instances) {
        let (instance_id, state) = (instance.instance_id().unwrap(), instance.state().unwrap());
        let name = state.name().unwrap();
        match name {
            InstanceStateName::Stopped => {
                // _ = client
                //     .start_instances()
                //     .instance_ids(instance_id)
                //     .send()
                //     .await
                //     .map_err(|e| e.to_string())?;
            }
            InstanceStateName::Running => {
                // let public_ip = instance.public_ip_address().unwrap();
                // return Ok(String::from(public_ip));
            }
            _ => {} // probably an in-between state
        }
    }

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
