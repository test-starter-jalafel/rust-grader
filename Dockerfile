FROM rust:1.85.0-slim

RUN apt-get update \
 && apt-get install jq -y \
 && apt-get autoremove -y \
 && apt-get install coreutils -y \
 && rm -rf /var/lib/apt/lists/*

COPY . /opt/test-runner
WORKDIR /opt/test-runner

ENTRYPOINT [ "/opt/test-runner/bin/run_autograding.sh" ]