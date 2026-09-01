FROM nginx:1.29-alpine@sha256:5616878291a2eed594aee8db4dade5878cf7edcb475e59193904b198d9b830de

ARG NGINX_CONFIG=deploy/nginx.static.conf
COPY ${NGINX_CONFIG} /etc/nginx/conf.d/default.conf

ARG BUNDLE=.fly-artifacts/web
COPY ${BUNDLE}/ /public/

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
