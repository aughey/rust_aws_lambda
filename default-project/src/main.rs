use aws_sdk_ec2::{
    model::{Instance, InstanceStateName},
    output::DescribeInstancesOutput,
    Client,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::Deserialize;
mod response;

/// This is a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    tags: Vec<Tag>,
}

#[derive(Deserialize, Debug)]
struct Tag {
    key: String,
    value: String,
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
        _ => "unknown",
    }
    .to_string()
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<Request>) -> Result<response::RunningInstance, Error> {
    function_handler_provision(&event.payload, &event.context.request_id).await
}

async fn function_handler_provision(
    command: &Request,
    _request_id: &str,
) -> Result<response::RunningInstance, Error> {
    let tags_requested = &command.tags;

    // There must be at least one tag
    if tags_requested.is_empty() {
        return Err("No tags provided".into());
    }

    let config = aws_config::from_env().load().await;

    let client = Client::new(&config);

    // Trying some crazy foo here
    // only spend 1 minute trying to find an instance
    let start = std::time::Instant::now();

    let mut actions = vec![];

    let instance = loop {
        if start.elapsed().as_secs() >= 60 {
            return Err("Timeout".into());
        }

        let instances = client
            .describe_instances()
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let reservations = instance_reservations(instances);

        // Filter the reservations to ones that include the requested tags
        let reservations = reservations.filter(|i| {
            // Get an iterator of the tags on this instance
            let instance_tags = i
                .tags()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|t| Some((t.key()?, t.value()?)))
                .collect::<Vec<_>>();

            // find a tag in requested_tags that is NOT in instance_tags
            let missing_tag = tags_requested
                .iter()
                .find(|t| !instance_tags.contains(&(t.key.as_str(), t.value.as_str())));

            // if there is a missing tag, this instance is not a match
            missing_tag.is_none()
        });

        let instances = reservations
            .filter_map(|i| {
                Some(response::Instance {
                    id: i.instance_id()?.into(),
                    state: instance_state_name(i.state()?.name()?),
                    ip: i.public_ip_address().map(|s| s.into()),
                })
            })
            .collect::<Vec<_>>();

        // if there are no instances, we fail
        if instances.is_empty() {
            tracing::error!("No instances found with tags: {:?}", tags_requested);
            tracing::info!("Possible instances: {:?}", instances);
            return Err(format!(
                "No instances found with tags: {:?} {:?}",
                tags_requested, instances
            )
            .into());
        }

        let running_instance = instances.iter().find(|i| i.state == "running");

        if let Some(instance) = running_instance {
            break instance.clone();
        }

        // Start all stopped instances
        for stopped_instance in instances.into_iter().filter(|i| i.state == "stopped") {
            let msg = format!("Starting instance {}", stopped_instance.id);
            tracing::info!("{}", msg);
            actions.push(msg);
            client
                .start_instances()
                .instance_ids(stopped_instance.id)
                .send()
                .await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(response::RunningInstance {
        ip: instance.ip.ok_or_else(|| "No IP address found".to_string())?,
        state: instance.state,
        id: instance.id,
        actions
    })
}

#[allow(dead_code)]
async fn get_running_instances(
    _command: &Request,
    request_id: &str,
) -> Result<response::Response, Error> {
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
        req_id: request_id.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_running_instances() {
        let request = Request { tags: vec![] };

        let response = get_running_instances(&request, "12345").await.unwrap();

        eprintln!("response: {:?}", response);
    }

    #[tokio::test]
    async fn test_provision() {
        let request = Request {
            tags: vec![Tag {
                key: "devcontainer".into(),
                value: "arm64.medium".into(),
            }],
        };

        let response = function_handler_provision(&request, "12345").await.unwrap();

        eprintln!("response: {:?}", response);
    }
}
