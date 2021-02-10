const { merge } = require("webpack-merge");
const common = require("./webpack.common.js");
const path = require("path");

const PROXY_PORT = process.env.PROXY_PORT || 9001

module.exports = merge(common, {
    mode: "development",
    devServer: {
        contentBase: path.join(__dirname, "dist"),
        compress: true,
        port: 9000,
        hot: true,
        historyApiFallback: true,
        disableHostCheck: true,
        proxy: [
            {
                context: ["/v2", "/login"],
                target: `http://localhost:${PROXY_PORT}`
            },
        ]
    },

    devtool: "inline-source-map",
});
