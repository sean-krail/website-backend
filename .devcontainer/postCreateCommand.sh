#!/bin/sh
pnpm install
curl -fsSL https://cargo-lambda.info/install.sh | sh
cargo install cargo-edit
pnpm format
pnpm build
pnpm test
pnpm cdk synth
