FROM node:lts-alpine AS build

WORKDIR /opt/axtell

COPY package.json package-lock.json tsconfig.json ./
COPY src src/
RUN npm ci
RUN npm run build

FROM node:lts-alpine AS deps

WORKDIR /opt/axtell
ENV NODE_ENV=production

COPY package.json package-lock.json ./
RUN npm ci

FROM node:lts-alpine AS test

WORKDIR /opt/axtell
ENV CI=1

COPY package.json package-lock.json ./
COPY src src/
COPY --from=deps /opt/axtell/node_modules node_modules/
ENTRYPOINT ["npm", "run", "test", "--", "--ci", "--detectOpenHandles"]

FROM node:lts-alpine AS run

WORKDIR /opt/axtell
ENV NODE_ENV=production

COPY package.json package-lock.json ./
COPY --from=build /opt/axtell/build build/
COPY --from=deps /opt/axtell/node_modules node_modules/
ENTRYPOINT ["npm", "start"]