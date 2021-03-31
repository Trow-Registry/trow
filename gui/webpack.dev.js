const { merge } = require("webpack-merge");
const common = require("./webpack.common.js");
const path = require("path");

const PROXY_PORT = process.env.PROXY_PORT || 9001;

module.exports = merge(common, {
  mode: "development",
  devServer: {
    port: 9000,
    historyApiFallback: true,
    disableHostCheck: true,
    //     https: true,
    //     contentBase: path.join(__dirname, "dist"),
    //     compress: true,
    //     hot: true,
  },
  devtool: "inline-source-map",
});
