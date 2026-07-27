FROM node:24-alpine AS builder

RUN corepack enable && corepack prepare pnpm@latest --activate
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
WORKDIR /app

COPY package.json pnpm-lock.yaml ./
RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install --frozen-lockfile

COPY . .
RUN pnpm build
# RUN pnpm --filter=frontend deploy out --prod --legacy

# FROM node:24-alpine AS runner
# RUN addgroup -S app && adduser -S app -G app
# WORKDIR /app
# COPY --from=builder --chown=app:app /app/out ./
# ENV NODE_ENV=production \
#     NODE_OPTIONS="--enable-source-maps"

# EXPOSE 4173
# USER app
# CMD ["npm", "run", "preview"]

FROM caddy:2-alpine AS runner

COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/build /usr/share/caddy

EXPOSE 80
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
