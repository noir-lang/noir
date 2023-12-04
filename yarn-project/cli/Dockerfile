FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/yarn-project-prod AS yarn-project-prod

# Set Tini as the default entrypoint, to handle ctrl-c etc.
# Why not just yarn start? About 1 second difference in startup time.
# At time of writing it still takes 1.7s to just get the help to print. Needs investigating.
RUN apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--", "node", "--no-warnings", "/usr/src/yarn-project/cli/dest/bin/index.js"]

# Setup cache volume.
ENV XDG_CACHE_HOME /cache
RUN mkdir /cache && chmod 777 /cache
VOLUME [ "/cache" ]

# Run as non-root user.
RUN corepack enable
RUN addgroup -S aztec && adduser -S aztec -G aztec
USER aztec
