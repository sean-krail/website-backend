import "source-map-support/register";
import { App } from "aws-cdk-lib";
import { CounterStack } from "../lib/counter-stack";

const app = new App();

new CounterStack(app, "CounterStack", {
  tags: {
    App: "website-backend",
    CloudFormationOwned: "true",
  },
});
