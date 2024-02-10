# [seankrail.dev](https://seankrail.dev/)'s backend

My personal website's backend built using TypeScript, AWS CDK, and Rust, hosted on AWS Lambda and DynamoDB, and continuously deployed by GitHub Actions.

Right now, it just provides an endpoint to increment a count, but plan to host more interesting features and experiments in the future.

## Development

There's a Lambda function written in Rust located at `functions/counter`. There you can run normal `cargo` commands to build, test, etc. You'll also need to install [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html) in your development environment. It's used to build a target for the Lambda environment.

```sh
cargo lambda build --release --arm64
```

This produces `target/lambda/counter/bootstrap`. This is referenced in our CounterStack at `lib/counter-stack.ts`. The `target/lambda/counter` directory is zipped, uploaded to S3, and then used to run the Lambda function.

The CDK components are split into three parts: `bin/`, `lib/`, `test/`. `bin/` holds the CDK entrypoint. It instantiates our CDK App and creates the CounterStack (referencing `lib/counter-stack.ts`). The CDK CLI generates the CloudFormation templates from it. `lib/` is where we define our CDK constructs and stacks referenced in `bin/website-backend.ts`. `test/` holds the unit tests for `lib/` resources.

For example, to build everything and synthesize the Cfn templates, run:

```sh
yarn build-function
yarn build
yarn cdk synth
```

## Updating

### Updating node via [mise](https://mise.jdx.dev/getting-started.html)

There's a `.mise.toml in this project root, so all you need to run is:

```sh
mise use node@20
node -v
npm -v
```

### Updating yarn

See https://yarnpkg.com/getting-started/install.

```sh
# assuming you have already run:
# corepack enable
yarn set version stable
```

### Updating npm packages

```sh
yarn up -i '*' '@*/*'
```

### Updating cargo crates

```sh
cargo update
cargo upgrade
```

## Useful CDK commands

The `cdk.json` file tells the CDK Toolkit how to execute your app.

- `yarn build` compile typescript to js
- `yarn watch` watch for changes and compile
- `yarn test` perform the jest unit tests
- `yarn cdk deploy` deploy this stack to your default AWS account/region
- `yarn cdk diff` compare deployed stack with current state
- `yarn cdk synth` emits the synthesized CloudFormation template
