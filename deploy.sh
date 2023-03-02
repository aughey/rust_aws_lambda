RUSTFLAGS='-C target-feature=+crt-static' cargo lambda build --bin default-project --release --x86-64 --compiler cargo --verbose
cargo lambda deploy --role `aws lambda get-function-configuration --function-name default-project | jq -r .Role` default-project
