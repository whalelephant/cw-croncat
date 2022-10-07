"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ts_codegen_1 = __importDefault(require("@cosmwasm/ts-codegen"));
(0, ts_codegen_1.default)({
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
