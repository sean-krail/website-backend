import { Duration, Stack, StackProps } from "aws-cdk-lib";
import { LambdaRestApi } from "aws-cdk-lib/aws-apigateway";
import {
  Certificate,
  CertificateValidation,
} from "aws-cdk-lib/aws-certificatemanager";
import { AttributeType, Table } from "aws-cdk-lib/aws-dynamodb";
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import {
  ARecord,
  PublicHostedZone,
  RecordTarget,
} from "aws-cdk-lib/aws-route53";
import { ApiGatewayDomain } from "aws-cdk-lib/aws-route53-targets";
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
      reservedConcurrentExecutions: 2,
      logRetention: RetentionDays.ONE_DAY,
    });
    table.grantReadWriteData(fn);

    const domainName = "api.seankrail.dev";
    const zone = new PublicHostedZone(this, "PublicHostedZone", {
      zoneName: domainName,
    });

    const certificate = new Certificate(this, "Certificate", {
      domainName,
      validation: CertificateValidation.fromDns(zone),
    });

    const api = new LambdaRestApi(this, "LambdaRestApi", {
      domainName: {
        domainName,
        certificate,
      },
      handler: fn,
      // Because we disable the proxy integration, our Lambda function must configure and return all CORS response headers
      proxy: false,
      disableExecuteApiEndpoint: true,
      deployOptions: {
        throttlingRateLimit: 2,
        throttlingBurstLimit: 5,
      },
    });
    const counter = api.root.addResource("count").addResource("{counter}");
    counter.addMethod("GET");
    counter.addMethod("POST");

    new ARecord(this, "AliasRecord", {
      zone,
      target: RecordTarget.fromAlias(new ApiGatewayDomain(api.domainName!)),
      ttl: Duration.hours(1),
    });
  }
}
