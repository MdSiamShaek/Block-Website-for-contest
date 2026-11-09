#!/bin/bash
# ========================================================
# CONTEST LOCKDOWN SCRIPT (Linux)
# Target: bapsoj.org (Chrome, Chromium, Brave)
# ========================================================

ALLOWED_SITE="bapsoj.org"
DIRS=(
    "/etc/opt/chrome/policies/managed"
    "/etc/chromium/policies/managed"
    "/etc/brave/policies/managed"
)

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root using sudo: sudo bash linux_lockdown.sh"
  exit 1
fi

echo "=========================================="
echo "   CONTEST LOCKDOWN TOOL (Linux)"
echo "   Allowed: https://$ALLOWED_SITE"
echo "=========================================="
echo "1. ENABLE Lockdown"
echo "2. DISABLE Lockdown"
read -p "Select an option (1 or 2): " CHOICE

if [ "$CHOICE" == "1" ]; then
    for DIR in "${DIRS[@]}"; do
        mkdir -p "$DIR"
        cat <<EOF > "$DIR/whitelist.json"
{
  "URLBlocklist": ["*"],
  "URLAllowlist": ["$ALLOWED_SITE", "*.$ALLOWED_SITE"],
  "DownloadRestrictions": 1
}
EOF
    done

    killall chrome google-chrome chromium brave brave-browser 2>/dev/null
    echo -e "\n[SUCCESS] Lockdown enabled! Only $ALLOWED_SITE is accessible."

elif [ "$CHOICE" == "2" ]; then
    for DIR in "${DIRS[@]}"; do
        rm -f "$DIR/whitelist.json"
    done
    echo -e "\n[SUCCESS] Lockdown removed! Normal browsing restored."
fi
