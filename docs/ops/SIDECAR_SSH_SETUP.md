# SSH Key Installation — Sidecar Access

We generated the sidecar SSH key on this workstation. It needs to be authorized on the PVE host and VM200.

## Your public key to install

```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEMh6iyt4J9cbU9n92woFfmL3laSnu3VsxnLapHX23O5 gzmo@sidecar
```

## Step 1: Authorize on PVE host (192.168.31.200)

This gives us `pct exec 101 -- ...` access to manage CT101:

```bash
# On the PVE host (ssh or web console):
echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEMh6iyt4J9cbU9n92woFfmL3laSnu3VsxnLapHX23O5 gzmo@sidecar' >> ~/.ssh/authorized_keys
```

## Step 2: Authorize on VM200 (192.168.31.110)

```bash
# SSH to VM200, then:
echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEMh6iyt4J9cbU9n92woFfmL3laSnu3VsxnLapHX23O5 gzmo@sidecar' >> ~/.ssh/authorized_keys
```

## Step 3: Verify from workstation

```bash
ssh -i ~/.ssh/id_sidecar_proxmox -o BatchMode=yes root@192.168.31.200 "hostname"
ssh -i ~/.ssh/id_sidecar_proxmox -o BatchMode=yes maximilian@192.168.31.110 "hostname"
```

## Step 4: Check daemon status on CT101

```bash
ssh -i ~/.ssh/id_sidecar_proxmox root@192.168.31.200 "pct exec 101 -- systemctl status gzmo-daemon"
ssh -i ~/.ssh/id_sidecar_proxmox root@192.168.31.200 "pct exec 101 -- ls /opt/gzmo/"
```
