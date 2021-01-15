const path = require("path");
const webpack = require("webpack");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");

const htmlTitle = "Trow. The Cloud Native Registry";
const devMode = process.env.NODE_ENV !== "production";
const ESLintPlugin = require("eslint-webpack-plugin");

/*
 * SplitChunksPlugin is enabled by default and replaced
 * deprecated CommonsChunkPlugin. It automatically identifies modules which
 * should be splitted of chunk by heuristics using module duplication count and
 * module category (i. e. node_modules). And splits the chunks…
 *
 * It is safe to remove "splitChunks" from the generated configuration
 * and was added as an educational example.
 *
 * https://webpack.js.org/plugins/split-chunks-plugin/
 *
 */

/*
 * We've enabled MiniCssExtractPlugin for you. This allows your app to
 * use css modules that will be moved into a separate CSS file instead of inside
 * one of your module entries!
 *
 * https://github.com/webpack-contrib/mini-css-extract-plugin
 *
 */

const MiniCssExtractPlugin = require("mini-css-extract-plugin");

/*
 * We've enabled TerserPlugin for you! This minifies your app
 * in order to load faster and run less javascript.
 *
 * https://github.com/webpack-contrib/terser-webpack-plugin
 *
 */

const TerserPlugin = require("terser-webpack-plugin");

module.exports = {
    entry: {
        app: "./src/App.js",
    },

    output: {
        filename: devMode ? "[name].[fullhash].js" : "[name].[chunkhash].js",
        path: path.resolve(__dirname, "dist"),
    },

    plugins: [
        new webpack.ProgressPlugin(),
        new webpack.DefinePlugin({
            "process.env.TROW_REGISTRY_URL": JSON.stringify(
                process.env.TROW_REGISTRY_URL
            ),
        }),
        new CleanWebpackPlugin(),
        new MiniCssExtractPlugin({
            filename: devMode
                ? "[name].[fullhash].css"
                : "[name].[chunkhash].css",
        }),
        new HtmlWebpackPlugin({
            title: htmlTitle,
            template: "html/index.html",
        }),
        new ESLintPlugin(),
    ],

    module: {
        rules: [
            {
                test: /\.(js|jsx|json)$/,
                include: [path.resolve(__dirname, "src")],
                loader: "babel-loader",
                options: {
                    presets: ["@babel/preset-env"],
                },
                exclude: /node_modules/,
            },
            {
                test: /\.(sa|sc|c)ss$/,
                use: [
                    devMode ? "style-loader" : MiniCssExtractPlugin.loader,
                    "css-loader",
                    // 'postcss-loader',
                    "sass-loader",
                ],
            },
            {
                test: /\.(png|svg|jpg|jpeg|gif)$/i,
                type: "asset/resource",
            },
            {
                test: /\.(woff|woff2|eot|ttf|otf)$/i,
                type: "asset/resource",
            },
        ],
    },

    optimization: {
        minimizer: [new TerserPlugin()],

        splitChunks: {
            cacheGroups: {
                vendors: {
                    priority: -10,
                    test: /[\\/]node_modules[\\/]/,
                },
            },

            chunks: "async",
            minChunks: 1,
            minSize: 30000,
            name: false,
        },
    },
};
