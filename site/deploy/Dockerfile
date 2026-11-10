FROM nginxinc/nginx-unprivileged:1.30.6-alpine@sha256:f5116ed4606f3eb015dd4a8e28cb9b351476dc0ad4bc6e9f93b19cec130cdf03

USER root
COPY nginx.conf /etc/nginx/nginx.conf
COPY security-headers.conf /etc/nginx/security-headers.conf
RUN rm -f /etc/nginx/conf.d/*
COPY --chown=0:0 html/ /usr/share/nginx/html/
RUN chmod -R a=rX /usr/share/nginx/html

ENTRYPOINT []
CMD ["nginx", "-g", "daemon off;"]
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s CMD wget -q -O /dev/null http://127.0.0.1:8080/__health || exit 1
USER 101
EXPOSE 8080
