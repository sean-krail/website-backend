import { Duration, Stack, StackProps } from "aws-cdk-lib";
import { AttributeType, Table } from "aws-cdk-lib/aws-dynamodb";
import {
  Architecture,
  Code,
  Function,
  FunctionUrlAuthType,
  Runtime,
} from "aws-cdk-lib/aws-lambda";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { Construct } from "constructs";

export class CounterStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const table = new Table(this, "Table", {
      tableName: "Counter",
      partitionKey: {
        name: "id",
        type: AttributeType.STRING,
      },
    });

    const fn = new Function(this, "Function", {
      // Directory with our `bootstrap` executable
      // You must run `yarn build-function` or `yarn build-all` first!
      code: Code.fromAsset("./functions/counter/target/lambda/counter"),
      runtime: Runtime.PROVIDED_AL2,
      architecture: Architecture.ARM_64,
      handler: "not.required",
      timeout: Duration.seconds(10),
      environment: {
        TABLE_NAME: table.tableName,
      },
      logRetention: RetentionDays.ONE_DAY,
    });
    table.grantReadWriteData(fn);
    fn.addFunctionUrl({
      authType: FunctionUrlAuthType.NONE,
      cors: {
        allowedOrigins: [
          "https://seankrail.dev",
          "https://krail.dev",
          "https://seankrail.com",
        ],
      },
    });
  }
}
