#!/usr/bin/env python3
"""Convert GZMO session log (.md) → session.json for distillation."""

import json
import re
import sys
import os


def parse_session_log(filepath):
    """Parse a session log into a structured format."""
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    # Extract session ID from the first SESSION marker
    session_id = "unknown"
    match = re.search(r"SESSION\s+([a-f0-9]+)", content)
    if match:
        session_id = match.group(1)

    # Split the content by role markers
    segments = re.split(
        r"(?=^###\s+(?:💬\s+USER|🧠\s+INTERNAL))",
        content,
        flags=re.MULTILINE
    )

    messages = []
    
    for segment in segments:
        if "💬 USER" in segment:
            marker_match = re.search(r"### 💬 USER — \d{2}:\d{2}:\d{2}\n", segment)
            if marker_match:
                text = segment[marker_match.end():].strip()
                messages.append({"role": "user", "content": text})
        elif "🧠 INTERNAL" in segment:
            marker_match = re.search(r"### 🧠 INTERNAL — \d{2}:\d{2}:\d{2}\n", segment)
            if marker_match:
                text = segment[marker_match.end():].strip()
                messages.append({"role": "assistant", "content": text})

    # Extract name from filename
    basename = os.path.basename(filepath)
    name = basename.replace(".md", "")
    
    # Add timestamp
    timestamp = "2026-07-15T11:47:00Z"
    if "2026-07-10" in filepath:
        timestamp = "2026-07-10T05:00:00Z"

    session = {
        "id": session_id,
        "name": name,
        "created_at": timestamp,
        "last_active_at": timestamp,
        "messages": messages
    }

    return session


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 convert_session_log.py <session_log.md>", file=sys.stderr)
        sys.exit(1)

    filepath = sys.argv[1]
    
    # Determine output path
    if len(sys.argv) > 2:
        output_path = sys.argv[2]
    else:
        output_path = filepath.replace(".md", ".json")

    session = parse_session_log(filepath)
    
    # Write JSON
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(session, f, indent=2, ensure_ascii=False)
    
    print(f"✓ Converted {len(session['messages'])} messages to {output_path}")


if __name__ == "__main__":
    main()
