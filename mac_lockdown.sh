#!/bin/bash
# ========================================================
# CONTEST LOCKDOWN SCRIPT (macOS)
# Target: bapsoj.org (Chrome, Brave, Edge)
# ========================================================

ALLOWED_SITE="bapsoj.org"
DOMAINS=("com.google.Chrome" "com.brave.Browser" "com.microsoft.edgemac")

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root using sudo: sudo bash mac_lockdown.sh"
  exit 1
fi

echo "=========================================="
echo "   CONTEST LOCKDOWN TOOL (macOS)"
echo "   Allowed: https://$ALLOWED_SITE"
echo "=========================================="
echo "1. ENABLE Lockdown"
echo "2. DISABLE Lockdown"
read -p "Select an option (1 or 2): " CHOICE

if [ "$CHOICE" == "1" ]; then
    for DOMAIN in "${DOMAINS[@]}"; do
        # Block all websites
        defaults write /Library/Preferences/$DOMAIN URLBlocklist -array "*"
        # Whitelist target domain and subdomains
        defaults write /Library/Preferences/$DOMAIN URLAllowlist -array "$ALLOWED_SITE" "*.$ALLOWED_SITE"
        # DownloadRestrictions: 1 = block dangerous downloads, 3 = block all
        defaults write /Library/Preferences/$DOMAIN DownloadRestrictions -int 1
    done

    # Force close running browsers so policies take effect immediately
    killall "Google Chrome" "Brave Browser" "Microsoft Edge" 2>/dev/null
    echo -e "\n[SUCCESS] Lockdown enabled! Only $ALLOWED_SITE is accessible in Chromium browsers."
    echo "Note: Ensure Safari is disabled or restricted via macOS Screen Time > Content Restrictions."

elif [ "$CHOICE" == "2" ]; then
    for DOMAIN in "${DOMAINS[@]}"; do
        defaults delete /Library/Preferences/$DOMAIN URLBlocklist 2>/dev/null
        defaults delete /Library/Preferences/$DOMAIN URLAllowlist 2>/dev/null
        defaults delete /Library/Preferences/$DOMAIN DownloadRestrictions 2>/dev/null
    done
    echo -e "\n[SUCCESS] Lockdown disabled! Normal internet browsing restored."
fi
