# syntax=docker/dockerfile:1

# ==============================================================================
# Multi-stage Dockerfile for Omnideps Website & Documentation
# ==============================================================================

# Stage 1: Build application
FROM node:22-alpine AS builder

WORKDIR /app

# Install dependencies needed for build
COPY package.json package-lock.json ./
RUN npm ci

# Copy application sources
COPY . .

# Build SvelteKit standalone bundle + Pagefind offline search index
RUN npm run build

# Remove development dependencies to keep runtime image small
RUN npm prune --production

# Stage 2: Minimal, secure production runtime
FROM node:22-alpine AS runner

WORKDIR /app

# Production environment variables
ENV NODE_ENV=production \
    PORT=3000 \
    HOST=0.0.0.0

# Copy production dependencies and standalone build output
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/build ./build

# Run as non-root user
USER node

# Default port
EXPOSE 3000

# Container healthcheck
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:3000/ || exit 1

# Launch the standalone server
CMD ["node", "build/index.js"]
