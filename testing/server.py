from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
import uvicorn
import random
import time

app = FastAPI()

# All HTTP methods routed through a single catch-all handler
@app.api_route("/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
async def catch_all(request: Request, path: str):
    # Randomly return 200 or 404 to simulate a real server
    status = random.choice([200, 200, 200, 404])

    print(
        f"[{time.strftime('%H:%M:%S')}] "
        f"{request.method:<8} "
        f"/{path:<30} "
        f"→ {status} "
        f"| UA: {request.headers.get('user-agent', 'unknown')[:60]}"
    )

    return JSONResponse(
        status_code=status,
        content={
            "method": request.method,
            "path":   f"/{path}",
            "status": status,
        }
    )

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8080, log_level="warning")
