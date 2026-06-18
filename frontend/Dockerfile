# syntax=docker/dockerfile:1

FROM node:20-slim AS dev
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 5173
CMD ["npm", "run", "dev", "--", "--host"]

# ---- prod build (documented, not used by docker-compose) ----
FROM dev AS build
RUN npm run build

FROM nginx:alpine AS prod
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
