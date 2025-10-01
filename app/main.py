# SPDX-License-Identifier: SSPL-1.0
from fastapi import FastAPI, UploadFile, Form, HTTPException
from fastapi.responses import JSONResponse, PlainTextResponse
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from typing import Optional
import yaml

from .parser_core import FGTParser

app = FastAPI(title="FGT Web Parser", version="0.1.0")
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"], allow_methods=["*"], allow_headers=["*"],
)

@app.get("/healthz")
def healthz():
    return {"ok": True}

@app.post("/api/parse")
async def parse_config(
    file: Optional[UploadFile] = None,
    text: Optional[str] = Form(None),
    format: Optional[str] = Form("json"),
):
    raw: Optional[str] = None
    if file:
        if file.content_type and "text" not in file.content_type and "octet-stream" not in file.content_type:
            raise HTTPException(status_code=400, detail="Unsupported content type")
        raw = (await file.read()).decode("utf-8", errors="replace")
    elif text:
        raw = text
    else:
        raise HTTPException(status_code=400, detail="Provide a file or text")

    parser = FGTParser()
    data = parser.parse_text(raw)

    if format == "yaml":
        return PlainTextResponse(yaml.safe_dump(data, sort_keys=False), media_type="text/yaml")
    return JSONResponse(content=data)

# Serve static UI from / (index.html, etc.)
app.mount("/", StaticFiles(directory="public", html=True), name="static")
