FROM node:24-alpine AS builder

RUN corepack enable && corepack prepare pnpm@latest --activate
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
WORKDIR /app

COPY package.json ./
RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install

ARG PUBLIC_API_URL
ENV PUBLIC_API_URL=${PUBLIC_API_URL}
ARG PUBLIC_IPFS_URL
ENV PUBLIC_IPFS_URL=${PUBLIC_IPFS_URL}
COPY . .
RUN pnpm build

FROM caddy:2-alpine AS runner

COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/build /usr/share/caddy

ARG PUBLIC_API_URL
ENV PUBLIC_API_URL=${PUBLIC_API_URL}
ARG PUBLIC_IPFS_URL
ENV PUBLIC_IPFS_URL=${PUBLIC_IPFS_URL}
EXPOSE 80
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
