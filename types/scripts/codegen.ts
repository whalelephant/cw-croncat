import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    {
      name: "CW-CRONCAT",
      dir: "../packages/cw-croncat-core/schema",
    },
    {
      name: "CW-RULES",
      dir: "../packages/cw-rules-core/schema",
    },
  ],
  outPath: "./contract",

  // options are completely optional ;)
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
    types: {
      enabled: true,
    },
    client: {
      enabled: true,
    },
    reactQuery: {
      enabled: false,
      optionalClient: true,
      version: "v4",
      mutations: true,
      queryKeys: true,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: false,
    },
  },
}).then(() => {
  console.log("✨ all done!");
});
