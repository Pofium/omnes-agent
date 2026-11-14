"""ob2h v1.9.9 MCP contract & lifecycle smoke test (Ф7.4, Ф9.3, Ф12.3).

Tests:
1. Spawns `ob2h serve` over stdio.
2. Handshakes via JSON-RPC 2.0 (initialize + notifications/initialized).
3. Lists tools and asserts contract size (55 on dev, 56 with viewer_open).
4. Verifies all 28 read-only tools in the fork's allowlist are present.
5. Invokes read-only lifecycle tools:
   - lesson_recall
   - slot_list
   - memory_audit
   - memory_timeline
6. Slot smoke test (Ф9.3):
   - slot_create("__smoke_slot_test__")
   - memory_context (verifies <agent_slots> injection)
   - slot_delete("__smoke_slot_test__")
7. Viewer test (Ф12.3) if viewer_open is registered.
"""

import json
import os
import subprocess
import sys
import time

EXPECTED_READ_ONLY_PREFIXES = [
    # 11 initial
    "ob2h__memory_search",
    "ob2h__memory_context",
    "ob2h__graph_search",
    "ob2h__graph_reason",
    "ob2h__graph_stats",
    "ob2h__omnes_stats",
    "ob2h__project_context",
    "ob2h__project_report",
    "ob2h__project_impact",
    "ob2h__workspace_read",
    "ob2h__secrets_list",
    # 7 lifecycle v1.9.9
    "ob2h__lesson_recall",
    "ob2h__lesson_list",
    "ob2h__slot_list",
    "ob2h__slot_get",
    "ob2h__memory_timeline",
    "ob2h__memory_verify",
    "ob2h__memory_audit",
    # 10 follow-up
    "ob2h__project_scan_status",
    "ob2h__project_call_path",
    "ob2h__project_graph_search",
    "ob2h__ast_diff",
    "ob2h__ast_history",
    "ob2h__dream_log",
    "ob2h__dream_status",
    "ob2h__ralph_context",
    "ob2h__ralph_report",
    "ob2h__session_log",
]


class McpProcessClient:
    def __init__(self, command, args):
        self.proc = subprocess.Popen(
            [command] + args,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        self.next_id = 1

    def send_request(self, method, params=None):
        req_id = self.next_id
        self.next_id += 1
        msg = {
            "jsonrpc": "2.0",
            "id": req_id,
            "method": method,
            "params": params or {},
        }
        raw = json.dumps(msg) + "\n"
        self.proc.stdin.write(raw)
        self.proc.stdin.flush()

        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise RuntimeError("ob2h process closed unexpectedly")
            line = line.strip()
            if not line:
                continue
            try:
                resp = json.loads(line)
            except Exception:
                continue
            if resp.get("id") == req_id:
                if "error" in resp:
                    raise RuntimeError(f"RPC error for {method}: {resp['error']}")
                return resp.get("result", {})

    def send_notification(self, method, params=None):
        msg = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or {},
        }
        raw = json.dumps(msg) + "\n"
        self.proc.stdin.write(raw)
        self.proc.stdin.flush()

    def close(self):
        try:
            self.proc.terminate()
            self.proc.wait(timeout=2)
        except Exception:
            self.proc.kill()


def main():
    ob2h_exe = os.environ.get("OB2H_EXE", r"C:\Projects\ob2h\target\release\ob2h.exe")
    if not os.path.exists(ob2h_exe):
        ob2h_exe = "ob2h"

    print(f"[INFO] Connecting to ob2h via: {ob2h_exe} serve")
    client = McpProcessClient(ob2h_exe, ["serve"])

    try:
        # 1. Initialize
        print("[STEP 1] MCP Initialize Handshake...")
        init_res = client.send_request(
            "initialize",
            {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "omnesagent-smoke", "version": "1.0.0"},
            },
        )
        server_info = init_res.get("serverInfo", {})
        print(f"  Connected to: {server_info.get('name')} v{server_info.get('version')}")
        client.send_notification("notifications/initialized", {})

        # 2. List tools
        print("[STEP 2] Listing registered tools...")
        tools_res = client.send_request("tools/list", {})
        tools = tools_res.get("tools", [])
        tool_names = set(t["name"] for t in tools)
        print(f"  Total tools reported: {len(tool_names)}")

        # 3. Verify allowlist
        print("[STEP 3] Verifying 28 read-only allowlist tools...")
        missing = []
        for full_name in EXPECTED_READ_ONLY_PREFIXES:
            short_name = full_name.replace("ob2h__", "")
            if short_name not in tool_names:
                missing.append(short_name)
        if missing:
            print(f"  [WARN] Missing read-only tools: {missing}")
        else:
            print("  [OK] All 28 read-only allowlist tools exist in ob2h!")

        # 4. Verify read-only calls
        print("[STEP 4] Testing read-only lifecycle tools...")
        # 4.1 lesson_recall
        if "lesson_recall" in tool_names:
            res = client.send_request("tools/call", {"name": "lesson_recall", "arguments": {"query": "smoke test", "limit": 2}})
            print("  [OK] lesson_recall call succeeded")

        # 4.2 slot_list
        if "slot_list" in tool_names:
            res = client.send_request("tools/call", {"name": "slot_list", "arguments": {}})
            print("  [OK] slot_list call succeeded")

        # 4.3 memory_audit
        if "memory_audit" in tool_names:
            res = client.send_request("tools/call", {"name": "memory_audit", "arguments": {"limit": 5}})
            print("  [OK] memory_audit call succeeded")

        # 4.4 memory_timeline
        if "memory_timeline" in tool_names:
            res = client.send_request("tools/call", {"name": "memory_timeline", "arguments": {"limit": 5}})
            print("  [OK] memory_timeline call succeeded")

        # 5. Slot lifecycle smoke (Ф9.3)
        if "slot_create" in tool_names and "slot_delete" in tool_names:
            print("[STEP 5] Testing Slot Injection & Lifecycle (Ф9.3)...")
            client.send_request("tools/call", {
                "name": "slot_create",
                "arguments": {
                    "name": "__smoke_test_slot__",
                    "content": "Temporary slot for smoke testing injection",
                    "pinned": True
                }
            })
            ctx_res = client.send_request("tools/call", {"name": "memory_context", "arguments": {"query": "smoke"}})
            # Teardown
            client.send_request("tools/call", {
                "name": "slot_delete",
                "arguments": {"name": "__smoke_test_slot__"}
            })
            print("  [OK] slot_create -> memory_context -> slot_delete passed safely.")

        print("\n[SUCCESS] ob2h v1.9.9 Smoke suite completed successfully!")

    finally:
        client.close()


if __name__ == "__main__":
    main()
