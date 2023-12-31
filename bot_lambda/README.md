Just for fun. Not maintained well.

## Prerequiste
cargo-lambda

The aws secrets manager in terraform directory.
It must contain the following.
username: string username of the truth social account
password: string password of the truth social account

## To build and run 
In one terminal

    cargo lambda watch

From another terminal

    cargo lambda invoke

## To build and deploy

    cargo lambda build --release --arm64 --output-format zip

This shall create the `target/lambda/src/bootstrap.zip` for deployment.
Then go to terraform infra directory to do the rest in terraform. This project doesn't setup anything for `cargo lambda deploy`
