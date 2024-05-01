########################################################################################################################
# Build wasi-sdk.
FROM ubuntu:noble AS wasi-sdk-build
RUN apt update && apt install -y \
    clang \
    cmake \
    ninja-build \
    git \
    cargo
RUN git clone --depth 1 --recursive --branch wasi-sdk-22 \
        https://github.com/WebAssembly/wasi-sdk.git
RUN mkdir -p /wasi-sdk/build/install/opt/wasi-sdk
WORKDIR /wasi-sdk
ENV MAKEFLAGS="-j$(nproc)"
RUN make build/llvm.BUILT
RUN make build/wasi-libc.BUILT
RUN make build/compiler-rt.BUILT
RUN make build/libcxx.BUILT
RUN make build/config.BUILT
RUN make build/version.BUILT
RUN mv build/install/opt/wasi-sdk /opt/wasi-sdk
FROM ubuntu:noble AS wasi-sdk
COPY --from=wasi-sdk-build /opt/wasi-sdk /opt/wasi-sdk

########################################################################################################################
# Build foundry.
FROM ubuntu:noble AS foundry
RUN apt update && apt install -y git cargo
RUN ulimit -n 65535 && \
    git clone --depth 1 --branch nightly-de33b6af53005037b463318d2628b5cfcaf39916 \
        https://github.com/foundry-rs/foundry.git && \
    cd foundry && cargo build --profile local && \
    mkdir -p /opt/foundry/bin && \
    for t in forge cast anvil chisel; do \
        mv ./target/local/$t /opt/foundry/bin/$t; \
        strip /opt/foundry/bin/$t; \
    done

########################################################################################################################
# This image contains *just* what's needed to perform a full build of the aztec project.
# It acts as the base image for all CI builds, and we build on it to produce a developer box.
FROM ubuntu:noble as build
RUN apt update && \
    apt install -y \
        # Utils
        curl \
        git \
        curl \
        wget \
        jq \
        gawk \
        unzip \
        netcat-openbsd \
        parallel \
        # C++ (clang=18, which we will move to. 16 is for current build.)
        build-essential \
        cmake \
        ninja-build \
        clang \
        clang-16 \
        clang-format-16 \
        libc++-dev \
        libomp-dev \
        doxygen \
        # Node (18.19.1)
        nodejs \
        npm \
        # Python (clang bindings for wasm bindgen.)
        python3 \
        python3-clang \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Install wasi-sdk.
COPY --from=aztecprotocol/wasi-sdk:22.0 /opt/wasi-sdk /opt/wasi-sdk

# Install foundry.
COPY --from=foundry /opt/foundry /opt/foundry
ENV PATH="/opt/foundry/bin:$PATH"

# Install rust and cross-compilers. Noir specifically uses 1.74.1.
# We add everyone write ownership so downstream boxes can write.
ENV RUSTUP_HOME=/opt/rust/rustup \
    CARGO_HOME=/opt/rust/cargo \
    PATH="/opt/rust/cargo/bin:$PATH"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.74.1 && \
    rustup target add wasm32-unknown-unknown wasm32-wasi aarch64-apple-darwin && \
    chmod -R a+w /opt/rust

# Install yq
RUN curl -L https://github.com/mikefarah/yq/releases/download/v4.42.1/yq_linux_$(dpkg --print-architecture) \
        -o /usr/local/bin/yq && chmod +x /usr/local/bin/yq

# Install yarn
RUN npm install --global yarn

# Install solhint
RUN npm install --global solhint

########################################################################################################################
# We want to produce downstream images: codespace, devbox and sysbox. This image is the base image for each.
# It contains a suite of tools that developers might use to develop aztec.
FROM build as basebox
RUN yes | unminimize

# Install stuff devs need.
RUN apt update && \
    apt install -y \
        zsh \
        fzf \
        libfuse2 \
        iproute2 \
        iputils-ping \
        telnet \
        lsb-release \
        tmux \
        vim \
        software-properties-common \
        gnupg \
        htop \
        cgroup-tools \
        neovim \
        sudo \
        clangd-16 \
        man \
        python3-blessed \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Install earthly.
RUN wget https://github.com/earthly/earthly/releases/latest/download/earthly-linux-$(dpkg --print-architecture) -O /usr/local/bin/earthly && \
    chmod +x /usr/local/bin/earthly

# Install gh (github cli).
RUN mkdir -p -m 755 /etc/apt/keyrings && wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg > /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt update \
    && apt install gh -y

# Install gt (graphite).
RUN npm install -g @withgraphite/graphite-cli@stable

# Install aws cli.
RUN curl "https://awscli.amazonaws.com/awscli-exe-linux-$(uname -m).zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && \
    ./aws/install --bin-dir /usr/local/bin --install-dir /usr/local/aws-cli --update && \
    rm -rf aws awscliv2.zip

# Install terraform.
RUN curl -fsSL https://releases.hashicorp.com/terraform/1.7.5/terraform_1.7.5_linux_$(dpkg --print-architecture).zip -o terraform.zip \
    && unzip terraform.zip -d /usr/local/bin \
    && chmod +x /usr/local/bin/terraform \
    && rm terraform.zip

# fzf seems to not install this file for some reason.
COPY ./key-bindings.zsh /usr/share/doc/fzf/examples/key-bindings.zsh

# Sets LANG explicitly. Ensures tmux shows unicode symbols.
# Sets RUSTUP_HOME.
# Adds foundry and cargo bin dirs to PATH.
COPY environment /etc/environment

# Cargo home and bin path should be set within users home dir at login.
RUN echo 'export CARGO_HOME="$HOME/.cargo"' >> /etc/zsh/zshenv
RUN echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> /etc/zsh/zshenv

# sudo group can sudo without password.
RUN echo '%sudo ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

########################################################################################################################
# This devbox container can be used to provide a full development environment.
#
# It can be used as a dev container:
# - Configuration in .devcontainer/devcontainer.json.
# - To run locally install "Dev Containers" plugin in vscode.
# - To run in GitHub codespaces, visit the repo in github, press '.', and open the terminal.
#
# It can be used independently:
# - The user should use the ./run.sh script to launch.
# - A persistent volume will be mounted to /home/aztec-dev.
# - It provides docker via the hosts docker instance, mounted at /var/lib/docker.sock.
# - It uses an entrypoint script at runtime to perform uid/gid alignment with the host and drop into user account.
FROM basebox as devbox
RUN apt install -y gosu
ENV TERM=xterm-256color
# Detect if the host machine is Mac, if so set an env var, and disable prompts vcs info for performance.
RUN <<EOF
echo '
if mount | grep -q /host_mark/Users; then
    export HOST_OSTYPE=darwin
    export PROMPT_LEAN_VCS=0
fi
' >> /etc/zsh/zshrc
EOF
# Create the user we'll run as and become the user.
RUN useradd --shell /bin/zsh -G sudo -m aztec-dev
USER aztec-dev
WORKDIR /home/aztec-dev
# Add dotfiles.
COPY --chown=aztec-dev:aztec-dev home .
# The .npmrc config is set to install global bins here, update PATH.
ENV PATH=/home/aztec-dev/.npm-global/bin:$PATH
# Need to ensure correct permissions, under some conditions these would otherwise be created by root.
RUN mkdir .vscode-server .npm-global .ssh
# Switch back to root. Gives option for root runtime adjustments before becoming aztec-dev.
USER root
# Use as entrypoint when running in an environment that requires uid/gid alignment (e.g. vanilla linux docker).
COPY ./entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/bin/zsh"]

########################################################################################################################
# This sysbox container can be used to provide a full development environment.
# It's more advanced than devbox in that it uses nestybox's sysbox container runtime to provide more of a vm experience.
# It's used primarily by internal aztec developers who have sysboxes running on a powerful underlying mainframe.
# It provides better isolation and security guarantees than a plain devbox.
FROM basebox AS sysbox

###################### START OF STOCK NESTYBOX SYSTEMD CONTAINER ###############################
#
# Systemd installation
#
RUN apt-get update &&                            \
    apt-get install -y --no-install-recommends   \
            systemd                              \
            systemd-sysv                         \
            libsystemd0                          \
            ca-certificates                      \
            dbus                                 \
            iptables                             \
            iproute2                             \
            kmod                                 \
            locales                              \
            sudo                                 \
            udev &&                              \
                                                 \
    # Prevents journald from reading kernel messages from /dev/kmsg
    echo "ReadKMsg=no" >> /etc/systemd/journald.conf &&               \
                                                                      \
    # Housekeeping
    apt-get clean -y &&                                               \
    rm -rf                                                            \
       /var/cache/debconf/*                                           \
       /var/lib/apt/lists/*                                           \
       /var/log/*                                                     \
       /tmp/*                                                         \
       /var/tmp/*                                                     \
       /usr/share/local/* &&                                          \
                                                                      \
    # Create default 'ubuntu/ubuntu' user
    echo "ubuntu:ubuntu" | chpasswd && adduser ubuntu sudo

# Disable systemd services/units that are unnecessary within a container.
RUN systemctl mask systemd-udevd.service \
                   systemd-udevd-kernel.socket \
                   systemd-udevd-control.socket \
                   systemd-modules-load.service \
                   sys-kernel-config.mount \
                   sys-kernel-debug.mount \
                   sys-kernel-tracing.mount \
                   e2scrub_reap.service

# Make use of stopsignal (instead of sigterm) to stop systemd containers.
STOPSIGNAL SIGRTMIN+3

# Set systemd as entrypoint.
ENTRYPOINT [ "/sbin/init", "--log-level=err" ]

###################### END OF STOCK NESTYBOX SYSTEMD CONTAINER ###############################

# Install docker.
RUN curl -fsSL https://get.docker.com -o get-docker.sh && sh get-docker.sh \
    # Add user "ubuntu" to the Docker group
    && usermod -a -G docker ubuntu
ADD https://raw.githubusercontent.com/docker/docker-ce/master/components/cli/contrib/completion/bash/docker /etc/bash_completion.d/docker.sh

# Install sshd.
RUN apt install --no-install-recommends -y openssh-server \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir /home/ubuntu/.ssh \
    && chown ubuntu:ubuntu /home/ubuntu/.ssh \
    && echo 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDagCvr/+CA1jmFaJf+e9+Kw6iwfhvaKOpfbGEl5zLgB+rum5L4Kga6Jow1gLQeMnAHfqc2IgpsU4t04c8PYApAt8AWNDL+KxMiFytfjKfJ2DZJA73CYkFnkfnMtU+ki+JG9dAHd6m7ShtCSzE5n6EDO2yWCVWQfqE3dcnpwrymSWkJYrbxzeOixiNZ4f1nD9ddvFvTWGB4l+et5SWgeIaYgJYDqTI2teRt9ytJiDGrCWXs9olHsCZOL6TEJPUQmNekwBkjMAZ4TmbBMjwbUlIxOpW2UxzlONcNn7IlRcGQg0Gdbkpo/zOlCNXsvacvnphDk5vKKaQj+aQiG916LU5P charlie@aztecprotocol.com' >> /home/ubuntu/.ssh/authorized_keys \
    && echo 'ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDb5OVc+9S9nXx3/34F7eLVXjoPgQ3YHSdlfhTb8WflAGmpKJTLxtAYngtDBvhKofH5HrjPPkBWxOHP9KOTo0jxUQSr0suMpggLLOHuIrCszJKXIVi7whnQ4p2RHyzyS2ANwmpxWZmYxfgamzYst9JIvQYJgAPjTFweKBsG/Lc03knJ/qgz9BHqDSZHweMTnhv1dJNhZRKy1Lxyl/CjXKF374i8qbzVWJMeDgLEH6C84vCeaH89KMmM9J0+T31uEqxzIhZxNmRz9v+x6cQAVJtGi9OIveGT9qUQwKXZsk6/zorzxV+NiIvTWHxIn9epX/FUjgUmb/jFvpbEjDkbIngj adomurad@localhost.localdomain' >> /home/ubuntu/.ssh/authorized_keys \
    && echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFKlUeOh9DyAL85NJ10LE+nyfi8oYm+CwxQ9JMaB6H+t root@mainframe' >> /home/ubuntu/.ssh/authorized_keys \
    && chown ubuntu:ubuntu /home/ubuntu/.ssh/authorized_keys

# Install google authenticator for setting up 2fa.
RUN apt update && apt install -y libpam-google-authenticator

EXPOSE 22
