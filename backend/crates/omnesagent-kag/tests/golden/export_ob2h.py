#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Экспорт эталонного набора символов/рёбер ob2h для golden dual-scan (задача Ф1.1).

Читает SQLite-БД ob2h (OB2H_DATA_DIR/ob2h.db) и выгружает нормализованные
ключи узлов и рёбер проекта в JSON. Формат совместим с
`omnesagent-kag/tests/test_golden_dual_scan.rs`.

Usage:
    python export_ob2h.py <ob2h.db> <project_id> <out.json> <ob2h_version>
"""

from __future__ import annotations

import json
import sqlite3
import sys


def main() -> int:
    if len(sys.argv) != 5:
        print(__doc__.strip(), file=sys.stderr)
        return 2

    db_path, project_id, out_path, ob2h_version = sys.argv[1:5]

    con = sqlite3.connect(db_path)
    cur = con.cursor()

    # node_id по первичному ключу — для сборки ключей рёбер.
    id_to_node: dict[int, str] = {}
    symbols: set[str] = set()
    for pk, node_id, prov in cur.execute(
        "SELECT id, node_id, provenance FROM graph_nodes WHERE project_id = ?",
        (project_id,),
    ):
        id_to_node[pk] = node_id
        if prov == "ast":
            symbols.add(node_id)

    # Узлы AST хранят provenance='ast'; рёбра — 'EXTRACTED'/'RESOLVED'/'INFERRED'.
    # Исключаем рёбра внешнего CodeGraph-индекса (provenance='codegraph').
    edges: list[str] = []
    for src, tgt, label in cur.execute(
        "SELECT source_id, target_id, label FROM graph_edges "
        "WHERE project_id = ? AND provenance != 'codegraph'",
        (project_id,),
    ):
        s = id_to_node.get(src)
        t = id_to_node.get(tgt)
        if s is None or t is None:
            continue
        edges.append(f"{s}\t{label}\t{t}")
    edges = sorted(set(edges))

    con.close()

    payload = {
        "ob2h_version": ob2h_version,
        "corpus": "corpus",
        "symbols": sorted(symbols),
        "edges": edges,
    }
    with open(out_path, "w", encoding="utf-8") as fh:
        json.dump(payload, fh, ensure_ascii=False, indent=2, sort_keys=False)
        fh.write("\n")

    print(f"wrote {out_path}: {len(symbols)} symbols, {len(edges)} edges")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
