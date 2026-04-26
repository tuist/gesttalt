# Gesttalt

Gesttalt is a decentralized publishing platform built with Elixir and Phoenix. ✍️🌍

## Why ✨

The web already showed us that publishing does not need to belong to a single platform. Blogs gave people space to think in public with depth and personality, and the Fediverse showed that social networks can be decentralized, portable, and interoperable. Gesttalt starts from the belief that these two ideas should meet.

We want something that goes beyond short posts without losing the networked nature of Mastodon. A publication should be able to feel like a magazine, a personal blog, a research journal, or a collective newsroom, while still participating in a broader social web.

We also want to separate the people who host publishing servers from the people who write on them. Not every writer, editor, or small publication should need to understand infrastructure, DNS, deployments, or server maintenance just to have a home on the web. In Gesttalt, hosting and publishing are related, but they are not the same responsibility.

And we want interoperability to be part of the foundation, not an afterthought. Gesttalt is meant to embrace Social Web standards so publications can connect across the Fediverse and beyond, while still being accessible from anywhere: the web, mobile apps, desktop clients, or automations. 📱

## Stack 🧰

- Elixir `1.19.5-otp-28`
- Erlang/OTP `28.5`
- Phoenix `1.8.5`
- PostgreSQL
- Plain CSS bundled with `esbuild`

The runtime toolchain is pinned in [`mise.toml`](./mise.toml). Phoenix itself is managed through [`mix.exs`](./mix.exs).

## Getting Started 🚀

1. Install the pinned toolchain with `mise install`
2. Install Phoenix bootstrap tooling with `mise tasks run setup`
3. Fetch deps and prepare the database with `mix setup`
4. Start the server with `mix phx.server`

Open the local URL printed by Phoenix in the terminal.

## Worktrees 🌳

This repository is configured like `tuist/tuist` for local Git worktrees.

Each checkout gets its own persisted dev instance suffix via Git metadata when available, and that suffix is used to derive unique `PORT`, `GESTTALT_DATABASE`, and `GESTTALT_TEST_DATABASE` values.

That means multiple worktrees can run Phoenix and tests side by side without colliding on ports or Postgres database names.

## Direction 🛰️

The next layer should introduce:

- publications and posts as the core writing model
- Fediverse discovery and delivery via ActivityPub, WebFinger, and NodeInfo
- IndieWeb publishing workflows like Micropub and Webmention
