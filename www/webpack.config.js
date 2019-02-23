/*eslint quotes: ["error", "double"]*/
/*eslint-env es6*/

const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

module.exports = {
    entry: "./bootstrap.ts",
    resolve: {
        extensions: [".tsx", ".ts", ".js", ".wasm"]
    },
    mode: "development",
    plugins: [
        new CopyWebpackPlugin(["index.html"]),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "../"),
            withTypeScript: true,
            forceMode: "production",
        }),
    ],
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: "ts-loader",
                exclude: /node_modules/
            },
            {
                test: /\.wasm$/,
                type: "webassembly/experimental"
            }
        ]
    },
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bootstrap.js",
    }
};
