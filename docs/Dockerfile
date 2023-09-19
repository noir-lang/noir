FROM node:18-alpine
RUN apk update

WORKDIR /usr/src

COPY . .

WORKDIR /usr/src/docs

RUN yarn && yarn build