
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse, PlainTextResponse
import uvicorn
import random
import time
import threading
from collections import defaultdict

app = FastAPI()

# Thread-safe statistics tracking
stats_lock = threading.Lock()
stats = {
    "total_calls": 0,
    "total_ok": 0,       # 2xx responses
    "total_errors": 0,   # 4xx/5xx responses
    "status_codes": defaultdict(int),
    "methods": defaultdict(int),
    "paths": defaultdict(int),
    "start_time": time.time(),
}


def print_summary():
    """Print a formatted summary of all calls received."""
    with stats_lock:
        elapsed = time.time() - stats["start_time"]
        print("\n" + "=" * 70)
        print("📊  SERVER CALL SUMMARY")
        print("=" * 70)
        print(f"  Uptime:            {elapsed:.1f} seconds")
        print(f"  Total Calls:       {stats['total_calls']}")
        print(f"  Total OK (2xx):    {stats['total_ok']}")
        print(f"  Total Errors:      {stats['total_errors']}")
        print("-" * 70)
        print("  Status Code Breakdown:")
        for code in sorted(stats["status_codes"].keys()):
            count = stats["status_codes"][code]
            pct = (count / stats["total_calls"] * 100) if stats["total_calls"] > 0 else 0
            bar = "█" * int(pct / 2)
            print(f"    {code}: {count:>6} ({pct:5.1f}%) {bar}")
        print("-" * 70)
        print("  HTTP Method Breakdown:")
        for method in sorted(stats["methods"].keys()):
            count = stats["methods"][method]
            print(f"    {method:<8}: {count:>6}")
        print("-" * 70)
        print("  Top 10 Paths:")
        sorted_paths = sorted(stats["paths"].items(), key=lambda x: x[1], reverse=True)[:10]
        for path, count in sorted_paths:
            print(f"    {path:<40} : {count:>6}")
        print("=" * 70 + "\n")


# Endpoint to get summary via HTTP
@app.get("/___summary")
async def get_summary():
    """Return the summary as a plain text response (also prints to console)."""
    print_summary()
    with stats_lock:
        elapsed = time.time() - stats["start_time"]
        summary_text = (
            f"Uptime: {elapsed:.1f}s | "
            f"Total: {stats['total_calls']} | "
            f"OK: {stats['total_ok']} | "
            f"Errors: {stats['total_errors']} | "
            f"Status Codes: {dict(stats['status_codes'])} | "
            f"Methods: {dict(stats['methods'])}"
        )
    return PlainTextResponse(content=summary_text)


# All HTTP methods routed through a single catch-all handler
@app.api_route("/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
async def catch_all(request: Request, path: str):
    # Randomly return various status codes to simulate a real server
    status = random.choice([200, 200, 200, 201, 301, 400, 404, 500])

    # Update statistics
    with stats_lock:
        stats["total_calls"] += 1
        stats["status_codes"][status] += 1
        stats["methods"][request.method] += 1
        stats["paths"][f"/{path}"] += 1
        if 200 <= status < 300:
            stats["total_ok"] += 1
        else:
            stats["total_errors"] += 1

        total = stats["total_calls"]

    # Log the request
    print(
        f"[{time.strftime('%H:%M:%S')}] "
        f"#{total:<6} "
        f"{request.method:<8} "
        f"/{path:<30} "
        f"→ {status} "
        f"| UA: {request.headers.get('user-agent', 'unknown')[:50]}"
    )

    # Print summary every 10 calls
    if total % 10 == 0:
        print_summary()

    return JSONResponse(
        status_code=status,
        content={
            "method": request.method,
            "path": f"/{path}",
            "status": status,
        }
    )


# Background thread to print summary periodically
def periodic_summary(interval=60):
    """Print summary every `interval` seconds."""
    while True:
        time.sleep(interval)
        if stats["total_calls"] > 0:
            print_summary()


if __name__ == "__main__":
    # Start periodic summary thread (prints every 60 seconds)
    summary_thread = threading.Thread(target=periodic_summary, args=(60,), daemon=True)
    summary_thread.start()

    print("🚀 Dossy Mock Server starting on http://0.0.0.0:8083")
    print("   - All paths return random status codes (200, 201, 301, 400, 404, 500)")
    print("   - Summary prints every 10 requests and every 60 seconds")
    print("   - Visit /___summary for on-demand summary")

    try:
        uvicorn.run(app, host="0.0.0.0", port=8083, log_level="warning")
    except KeyboardInterrupt:
        print("\n\n🛑 Server shutting down...")
        print_summary()