#!/usr/bin/env bash
# Setup script for Oh My Pi agent Telegram integration
# This script configures the OpenClaw workspace for Telegram integration
# within the GZMO ecosystem.

set -euo pipefail

# Ensure we're in the GZMO directory
cd /home/gzmo/github-clone/GZMO

# Create the OpenClaw workspace directory if it doesn't exist
mkdir -p ~/.openclaw/workspace

# Copy the necessary files to the workspace
cp config/openclaw-workspace/SOUL.md ~/.openclaw/workspace/
cp config/openclaw-workspace/IDENTITY.md ~/.openclaw/workspace/
cp config/openclaw-workspace/AGENTS.md ~/.openclaw/workspace/
cp config/openclaw-workspace/LIVING_ATTACH.md ~/.openclaw/workspace/

# Copy the takeaway script to the workspace bin directory
mkdir -p ~/.openclaw/workspace/bin
cp scripts/openclaw-takeaway.sh ~/.openclaw/workspace/bin/
chmod +x ~/.openclaw/workspace/bin/openclaw-takeaway.sh

# Create the skills directory and copy the character skill
mkdir -p ~/.openclaw/workspace/skills
cp -r config/openclaw-workspace/skills/character ~/.openclaw/workspace/skills/

# Update permissions for the copied files
chmod -R 644 ~/.openclaw/workspace/*
chmod -R 755 ~/.openclaw/workspace/bin
chmod -R 755 ~/.openclaw/workspace/skills

# Verify the setup
echo "Oh My Pi Telegram integration setup complete!"
echo "Files created in ~/.openclaw/workspace:"
ls -la ~/.openclaw/workspace/

# Display the MCP configuration that will be used
echo ""
echo "MCP Configuration for gzmo-living:"
./scripts/emit-living-mcp-fragment.sh --format json

# Display the OpenClaw workspace structure
echo ""
echo "OpenClaw workspace structure:"
find ~/.openclaw/workspace -type f -o -type d | sort