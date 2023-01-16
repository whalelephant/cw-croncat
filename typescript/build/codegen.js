"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const ts_codegen_1 = __importDefault(require("@cosmwasm/ts-codegen"));
const path_1 = __importDefault(require("path"));
const fs_1 = __importDefault(require("fs"));
var OutputType;
(function (OutputType) {
    OutputType["contracts"] = "contracts";
    OutputType["croncat"] = "cw-croncat";
    OutputType["rules"] = "cw-rules";
    OutputType["packages"] = "packages";
})(OutputType || (OutputType = {}));
const CONTRACTS_OUTPUT_DIR = ".";
const DEFAULT_CONFIG = {
    schemaRoots: [
        {
            name: OutputType.contracts,
            paths: [`../${OutputType.contracts}`],
            outputName: OutputType.contracts,
            outputDir: CONTRACTS_OUTPUT_DIR,
        },
        {
            name: OutputType.packages,
            paths: [`../${OutputType.packages}`],
            outputName: OutputType.packages,
            outputDir: CONTRACTS_OUTPUT_DIR,
        },
    ],
};
function generateTs(spec) {
    return __awaiter(this, void 0, void 0, function* () {
        const out = `${spec.outputPath}/${spec.outputType}/${spec.contractName}`;
        const name = spec.contractName;
        console.log(spec.schemaDir);
        return yield (0, ts_codegen_1.default)({
            contracts: [
                {
                    name: `${name}`,
                    dir: `${spec.schemaDir}`,
                },
            ],
            outPath: `./${OutputType.contracts}/${name}`,
        }).then(() => {
            console.log(`${name} done!`);
        });
    });
}
function getSchemaDirectories(rootDir) {
    return new Promise((resolve, _reject) => {
        const directories = [];
        // get all the schema directories in all the root dir
        fs_1.default.readdir(rootDir, (err, dirEntries) => {
            if (err) {
                console.error(err);
                return;
            }
            if (!dirEntries) {
                console.warn(`no entries found in ${rootDir}`);
                resolve([]);
                return;
            }
            dirEntries.forEach((entry) => {
                try {
                    const schemaDir = path_1.default.resolve(rootDir, entry, "schema");
                    if (fs_1.default.existsSync(schemaDir) &&
                        fs_1.default.lstatSync(schemaDir).isDirectory()) {
                        directories.push([schemaDir.replaceAll("\\", "/"), entry]);
                    }
                }
                catch (e) {
                    console.warn(e);
                }
            });
            resolve(directories);
        });
    });
}
function main() {
    var _a;
    return __awaiter(this, void 0, void 0, function* () {
        let config = Object.assign({}, DEFAULT_CONFIG);
        const compilationSpecs = [];
        console.log("Calculating generation specs...");
        for (const root of config.schemaRoots) {
            const { name, paths, outputName, outputDir } = root;
            for (const path of paths) {
                const schemaDirectories = yield getSchemaDirectories(path);
                console.log(schemaDirectories);
                for (const [directory, contractName] of schemaDirectories) {
                    compilationSpecs.push({
                        contractName: contractName,
                        schemaDir: directory,
                        outputPath: outputDir,
                        outputType: outputName,
                    });
                }
            }
        }
        console.log(`code generating for ${(_a = compilationSpecs === null || compilationSpecs === void 0 ? void 0 : compilationSpecs.length) !== null && _a !== void 0 ? _a : 0} specs...`);
        const codegenResponses = [];
        for (const spec of compilationSpecs) {
            codegenResponses.push(generateTs(spec));
        }
        yield Promise.all(codegenResponses);
        console.log(`code generation complete`);
    });
}
main();
