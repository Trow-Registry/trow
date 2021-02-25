const express = require("express");
const { createProxyMiddleware } = require("http-proxy-middleware");
const morgan = require("morgan");

const TROW_REGISTRY_URL =
  process.env.TROW_REGISTRY_URL || "https://trow.local:8443";

const PROXY_PORT = Number.isInteger(process.env.PROXY_PORT)
  ? parseInt(process.env.PROXY_PORT)
  : 9001;

// proxy middleware options
const options = {
  target: TROW_REGISTRY_URL, // target host
  changeOrigin: true,
  secure: false,
};

// create proxy
const trowProxy = createProxyMiddleware(options);

const app = express();

// add basic logging
app.use(morgan("dev"));

// mount proxy
app.use("/v2", trowProxy);
app.use("/login", trowProxy);

app.listen(PROXY_PORT);
