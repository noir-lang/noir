FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project AS builder

WORKDIR /usr/src/yarn-project/aztec-faucet

# Productionify. See comment in yarn-project-base/Dockerfile.
RUN yarn cache clean && yarn workspaces focus --production

# Create final, minimal size image.
FROM node:18-alpine
COPY --from=builder /usr/src/ /usr/src/
WORKDIR /usr/src/yarn-project/aztec-faucet
ENTRYPOINT ["yarn"]
CMD [ "start" ]
EXPOSE 8080
