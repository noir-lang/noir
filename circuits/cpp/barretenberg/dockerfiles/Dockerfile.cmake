FROM ubuntu:latest
RUN apt-get update && apt-get install -y build-essential wget git libssl-dev
RUN wget https://cmake.org/files/v3.16/cmake-3.16.5.tar.gz \
  && tar zxfv cmake-3.16.5.tar.gz \
  && cd cmake-3.16.5 \
  && ./bootstrap \
  && make -j$(nproc) \
  && make install \
  && cd .. \
  && rm -rf cmake*