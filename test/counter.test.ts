import { App } from "aws-cdk-lib";
import { Template } from "aws-cdk-lib/assertions";
import { CounterStack } from "../lib/counter-stack";

test("DynamoDB Table is created", () => {
  const app = new App();
  const stack = new CounterStack(app, "TestCounterStack");

  const template = Template.fromStack(stack);

  template.hasResourceProperties("AWS::DynamoDB::Table", {
    TableName: "Counter",
  });
});

test("Lambda Function is created and references DynamoDB Table", () => {
  const app = new App();
  const stack = new CounterStack(app, "TestCounterStack");

  const template = Template.fromStack(stack);

  const tableRef = Object.keys(
    template.findResources("AWS::DynamoDB::Table", {
      Properties: {
        TableName: "Counter",
      },
    })
  )[0];
  template.hasResourceProperties("AWS::Lambda::Function", {
    Runtime: "provided.al2",
    Environment: {
      Variables: {
        TABLE_NAME: {
          Ref: tableRef,
        },
      },
    },
  });
});

test("Lambda Function Url is created", () => {
  const app = new App();
  const stack = new CounterStack(app, "TestCounterStack");

  const template = Template.fromStack(stack);

  template.hasResourceProperties("AWS::Lambda::Url", {
    AuthType: "NONE",
  });
});

test("Lambda Function for CloudWatch logging is created", () => {
  const app = new App();
  const stack = new CounterStack(app, "TestCounterStack");

  const template = Template.fromStack(stack);

  template.hasResourceProperties("AWS::Lambda::Function", {
    Handler: "index.handler",
  });
});
