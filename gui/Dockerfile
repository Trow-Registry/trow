FROM --platform=${BUILDPLATFORM:-linux/amd64} node:alpine  as builder
WORKDIR /app
COPY package.json .
COPY yarn.lock .
RUN yarn 
COPY . .
RUN yarn build 

FROM --platform=${BUILDPLATFORM:-linux/amd64} nginx 
COPY --from=builder /app/dist /usr/share/nginx/html
CMD ["nginx", "-g", "daemon off;"]
