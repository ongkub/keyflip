# KeyFlip problem-report receiver: standard-library Python, runs as an unprivileged user.
FROM python:3.12-slim
WORKDIR /app
COPY feedback_server.py /app/feedback_server.py
# Docker copies this directory's ownership into a new named volume, so the unprivileged user can write the DB.
RUN mkdir -p /data && chown 65534:65534 /data
USER 65534:65534
ENV KEYFLIP_DB=/data/reports.db \
    KEYFLIP_BIND=0.0.0.0:8787 \
    PYTHONUNBUFFERED=1
EXPOSE 8787
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
  CMD python -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8787/healthz', timeout=2)"
CMD ["python", "/app/feedback_server.py"]
