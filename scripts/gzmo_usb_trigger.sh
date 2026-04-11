#!/usr/bin/env bash
# ==============================================================================
# GZMO USB Auto-Trigger
# Fired by udev when the SanDisk Phantom Drive is physically inserted.
# Decrypts, mounts, and spawns boot.sh in a visible terminal.
# ==============================================================================

set -euo pipefail

LUKS_KEY="/home/maximilian-wruhs/.gemini/antigravity/brain/48a649b9-7302-41c2-89a2-c94a0be41f58/gzmo_luks_key.txt"
MAPPER_NAME="gzmo_crypt"
MOUNT_POINT="/mnt/gzmo_usb"
LOG="/tmp/gzmo_trigger.log"

exec >> "$LOG" 2>&1
echo "$(date): GZMO USB trigger fired"

# Wait for kernel to settle partition nodes
sleep 3

# Find the LUKS partition on the SanDisk
PART=$(lsblk -lnpo NAME,TYPE /dev/sdb 2>/dev/null | awk '$2=="part"{print $1; exit}')
if [ -z "$PART" ]; then
    echo "$(date): No partition found on /dev/sdb"
    exit 1
fi

# Decrypt
if [ ! -e "/dev/mapper/$MAPPER_NAME" ]; then
    cryptsetup luksOpen "$PART" "$MAPPER_NAME" --key-file "$LUKS_KEY"
    echo "$(date): LUKS opened on $PART"
fi

# Mount
mkdir -p "$MOUNT_POINT"
if ! mountpoint -q "$MOUNT_POINT"; then
    mount "/dev/mapper/$MAPPER_NAME" "$MOUNT_POINT"
    echo "$(date): Mounted at $MOUNT_POINT"
fi

# Launch boot.sh in a visible terminal for the logged-in user
DISPLAY_USER="maximilian-wruhs"
export DISPLAY=:0
export XAUTHORITY="/home/$DISPLAY_USER/.Xauthority"

# Try gnome-terminal first, fall back to xterm
if command -v gnome-terminal &>/dev/null; then
    su - "$DISPLAY_USER" -c "DISPLAY=:0 gnome-terminal --title='GZMO Phantom Drive' -- bash -c 'cd $MOUNT_POINT && ./boot.sh; exec bash'"
elif command -v xterm &>/dev/null; then
    su - "$DISPLAY_USER" -c "DISPLAY=:0 xterm -title 'GZMO Phantom Drive' -e 'cd $MOUNT_POINT && ./boot.sh; exec bash'"
else
    echo "$(date): No terminal emulator found. Manual launch required."
    exit 1
fi

echo "$(date): GZMO launched successfully"
