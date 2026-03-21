import "source-map-support/register.js";
import { App } from "aws-cdk-lib";
import { CounterStack } from "../lib/counter-stack.js";

const app = new App();

new CounterStack(app, "CounterStack", {
  tags: {
    App: "website-backend",
    CloudFormationOwned: "true",
  },
});
