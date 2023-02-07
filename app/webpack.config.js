const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
    mode: "development",
    entry: "./bootstrap.js",
    output: {
        path: dist,
        filename: "bootstrap.js"
    },
    devServer: {
        contentBase: dist,
    },
    plugins: [
        new CopyPlugin([
            "index.html",
            "node_modules/xterm/css/xterm.css",
            "node_modules/clusterize.js/clusterize.css"
        ]),
    ]
};
