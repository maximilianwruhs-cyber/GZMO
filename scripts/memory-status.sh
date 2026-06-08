#!/usr/bin/env bash
# Quick status script for SQLite and Qdrant memory layers

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
python3 -c "import sqlite3, json, urllib.request; c=sqlite3.connect('$DIR/../data/vault.db');
for label, q in [('vault', 'SELECT COUNT(*) FROM semantic_vault'), ('honeypot', 'SELECT COUNT(*) FROM honeypot WHERE is_latest=1'), ('honeypot_fts', 'SELECT COUNT(*) FROM honeypot_fts')]:
    try: print(label, c.execute(q).fetchone()[0])
    except Exception as e: print(label, 'ERR', e)
for coll in ('honeypot', 'knowledge'):
    try:
        d=json.load(urllib.request.urlopen(f'http://192.168.31.202:6333/collections/{coll}'))
        print('qdrant', coll, d['result']['points_count'])
    except Exception as e: print('qdrant', coll, e)"
