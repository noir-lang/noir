---
title: Codespaces
sidebar_position: 0
draft: true
---

All machines are different, and you may not want to run the sandbox locally (for example when using Windows). We thought about you exactly ❤️ 

[Codespaces](https://github.com/features/codespaces) are a quick way to develop: they provision a remote machine with all tooling you need for Aztec in just a few minutes. We're big fans, so we prepared some prebuilt images to make it easier and faster.

Just choose a boilerplate and click "create new codespace":

[![One-Click React Starter](/img/codespaces_badges/react_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Freact%2Fdevcontainer.json) [![One-Click HTML/TS Starter](/img/codespaces_badges/vanilla_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Fvanilla%2Fdevcontainer.json) [![One-Click Token Starter](/img/codespaces_badges/token_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Ftoken%2Fdevcontainer.json)

This creates a codespace with a prebuilt image containing one of the "Aztec Boxes" and a development network (sandbox). 
- You can develop directly on the codespace, push it to a repo, make yourself at home.
- You can also just use the sandbox that comes with it. The URL will be logged, you just need to use it as your `PXE_URL`.

You can then start, stop, or see the logs of your sandbox just by calling `sandbox` or `npx aztec-app sandbox`. Run `sandbox -h` for a list of commands.

## More about codespaces

Codespaces are way more powerful than you may initially think. For example, you can connect your local `vscode` to a remote codespace, for a fully contained development environment that doesn't use any of your computer resources!

Visit the [codespaces documentation](https://docs.github.com/en/codespaces/overview) for more specific documentation around codespaces.
