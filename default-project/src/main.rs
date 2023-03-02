use aws_sdk_ec2::{
    model::{Instance, InstanceStateName},
    output::DescribeInstancesOutput,
    Client,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize};
mod response;

/// This is a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    command: String,
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

fn instance_state_name(state: &InstanceStateName) -> String {
    match state {
        InstanceStateName::Pending => "pending",
        InstanceStateName::Running => "running",
        InstanceStateName::ShuttingDown => "shutting-down",
        InstanceStateName::Terminated => "terminated",
        InstanceStateName::Stopping => "stopping",
        InstanceStateName::Stopped => "stopped",
        _ => "unknown"
    }.to_string()
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<Request>) -> Result<response::Response, Error> {
    // Extract some useful info from the request
    let _command = event.payload.command;

    let config = aws_config::from_env().load().await;

    let client = Client::new(&config);

    let instances = client
        .describe_instances()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let instances = instance_reservations(instances).filter_map(|i| {
        let id = i.instance_id()?;
        let state = i.state()?.name()?;
        let ip = i.public_ip_address();
        Some(response::Instance {
            id: String::from(id),
            state: instance_state_name(state),
            ip: ip.map(|s| String::from(s)),
        })
    });

    let response = response::Response {
        req_id: event.context.request_id,
        instances: instances.collect(),
    };

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
