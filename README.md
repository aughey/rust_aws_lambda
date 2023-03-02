# rust_aws_lambda

This is a sample project to demonstrate using aws lambda with rust for API calls.

- Guide: https://github.com/awslabs/aws-lambda-rust-runtime#2-deploying-the-binary-to-aws-lambda
- Deploying aws lambda: https://docs.aws.amazon.com/sdk-for-rust/latest/dg/lambda.html

```
cargo install cargo-lambda
RUSTFLAGS='-C target-feature=+crt-static' cargo lambda build --bin default-project --release --x86-64 --compiler cargo --verbose
cargo lambda deploy --role arn:aws:lambda:us-east-2:801171132372:function:default-project/prototype_aws_lambda-role-sg9lhk6x default-project
cargo lambda deploy --role `aws lambda get-function-configuration --function-name default-project | jq -r .Role` default-project
```

From: https://www.cargo-lambda.info/guide/getting-started.html#step-2-create-a-new-project

```
cargo lambda new new-lambda-project \
    && cd new-lambda-project

cargo lambda new -http new-lambda-project \
    && cd new-lambda-project
```

# Plans

2 projects, an http project and a project that isn't setup to respond to any specific event.