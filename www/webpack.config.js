const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
    publicPath: './',
  },
  mode: process.env.NODE_ENV === 'production' ? 'production' : 'development',
  experiments: {
    asyncWebAssembly: true,
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        { from: "index.html" }
      ]
    })
  ],
  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'),
    },
    compress: true,
    port: 8080,
  },
};
