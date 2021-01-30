const path = require('path');
const HtmlWebpackPlugin = require("html-webpack-plugin");
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

module.exports = {
    entry: './index.tsx',
    devtool: 'source-map',
    mode: 'production',
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'awesome-typescript-loader',
                exclude: /node_modules/,
            },
            {
                test:/\.css$/,
                use: [MiniCssExtractPlugin.loader, 'css-loader']
            },
        ],
    },
    resolve: {
        extensions: [ '.tsx', '.ts', '.js' ],
    },
    output: {
        filename: 'bm.js',
        path: path.resolve(__dirname, '..', 'www'),
    },
    plugins: [
        new MiniCssExtractPlugin({
            filename: "bm.css",
        }),
        new HtmlWebpackPlugin({
            template: path.resolve(__dirname, "index.html"),
        }),
    ]
};
