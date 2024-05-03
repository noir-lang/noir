VERSION 0.8

acir-tests:
    FROM ../build-images+build
    WORKDIR /usr/src/barretenberg
    COPY ./acir_tests .
    SAVE ARTIFACT ./*

sol:
  FROM ../build-images+build
  WORKDIR /usr/src/barretenberg
  COPY ./sol .
  SAVE ARTIFACT ./*

