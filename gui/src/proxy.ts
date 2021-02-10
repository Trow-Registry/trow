import express from 'express'
import { createProxyMiddleware} from 'http-proxy-middleware';
import morgan from 'morgan'

const TROW_REGISTRY_URL: string = process.env.TROW_REGISTRY_URL || 'https://trow.local:8443'

const PROXY_PORT: number =  Number.isInteger(process.env.PROXY_PORT) ? parseInt(process.env.PROXY_PORT) : 9001;

// proxy middleware options
const options = {
  target: TROW_REGISTRY_URL, // target host
  changeOrigin: true, 
  secure: false
};

// create proxy
const trowProxy = createProxyMiddleware(options);

const app = express();

// add basic logging
app.use(morgan('dev'))

// mount proxy
app.use('/v2', trowProxy);
app.use('/login', trowProxy);

app.listen(PROXY_PORT);