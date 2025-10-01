# SPDX-License-Identifier: SSPL-1.0
FROM python:3.11-slim AS app
WORKDIR /app
COPY pyproject.toml .
RUN pip install --no-cache-dir fastapi uvicorn python-multipart pyyaml
COPY app ./app
COPY public ./public
EXPOSE 8080
CMD ["uvicorn", "app.main:app", "--host=0.0.0.0", "--port=8080", "--proxy-headers", "--forwarded-allow-ips", "*"]
