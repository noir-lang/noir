FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS builder
WORKDIR /usr/src
COPY . .
WORKDIR /usr/src/yarn-project
RUN yarn build
WORKDIR /usr/src/docs
RUN yarn && yarn build