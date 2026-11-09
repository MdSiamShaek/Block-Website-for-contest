# ========================================================
# CONTEST LOCKDOWN SCRIPT (PowerShell)
# Target: bapsoj.org
# ========================================================

$AllowedSite = "bapsoj.org"

# SHA-256 Hash of your secret password (Default: MySecretPass123!)
$StoredHash = "99A4E22854E630750D579F4C987DC57795EC6219DF8EE7939BC8BF5F61431B32"

# Ensure the script is running as Administrator
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Start-Process -FilePath PowerShell.exe -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -Verb RunAs
    exit
}

function Test-AdminPassword {
    $SecureInput = Read-Host "Enter Admin Unlock Password" -AsSecureString
    $BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($SecureInput)
    $PlainInput = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)
    [System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($BSTR)

    $Bytes = [System.Text.Encoding]::UTF8.GetBytes($PlainInput)
    $HashBytes = [System.Security.Cryptography.SHA256]::Create().ComputeHash($Bytes)
    $InputHash = [System.BitConverter]::ToString($HashBytes).Replace("-","")

    return ($InputHash -eq $StoredHash)
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   CONTEST LOCKDOWN TOOL ($AllowedSite)" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "1. ENABLE Lockdown (Allow ONLY https://$AllowedSite)"
Write-Host "2. DISABLE Lockdown (Requires Password)"
$Choice = Read-Host "Select an option (1 or 2)"

$ChromiumPaths = @(
    "HKLM:\SOFTWARE\Policies\Google\Chrome",
    "HKLM:\SOFTWARE\Policies\Microsoft\Edge",
    "HKLM:\SOFTWARE\Policies\BraveSoftware\Brave"
)

$FirefoxPath = "HKLM:\SOFTWARE\Policies\Mozilla\Firefox"

$BlockedExes = @(
    "opera.exe", "vivaldi.exe", "tor.exe", "waterfox.exe", "maxthon.exe",
    "IronPortable.exe", "FirefoxPortable.exe", "GoogleChromePortable.exe",
    "OperaPortable.exe", "BravePortable.exe"
)

if ($Choice -eq "1") {
    # 1. Lock down Chromium Browsers (Chrome, Edge, Brave)
    foreach ($Path in $ChromiumPaths) {
        Remove-Item -Path "$Path\URLBlocklist", "$Path\URLAllowlist" -Recurse -ErrorAction SilentlyContinue
        New-Item -Path "$Path\URLBlocklist" -Force | Out-Null
        New-Item -Path "$Path\URLAllowlist" -Force | Out-Null

        Set-ItemProperty -Path "$Path\URLBlocklist" -Name "1" -Value "*"
        Set-ItemProperty -Path "$Path\URLAllowlist" -Name "1" -Value $AllowedSite
        Set-ItemProperty -Path "$Path\URLAllowlist" -Name "2" -Value "*.$AllowedSite"
        # DownloadRestrictions: 1 = Allow safe files (.txt, .zip), block .exe/.msi
        Set-ItemProperty -Path $Path -Name "DownloadRestrictions" -Value 1 -Type DWord
    }

    # 2. Lock down Mozilla Firefox
    Remove-Item -Path "$FirefoxPath\WebsiteFilter" -Recurse -ErrorAction SilentlyContinue
    New-Item -Path "$FirefoxPath\WebsiteFilter\Block" -Force | Out-Null
    New-Item -Path "$FirefoxPath\WebsiteFilter\Exceptions" -Force | Out-Null

    Set-ItemProperty -Path $FirefoxPath -Name "DisableDownloads" -Value 1 -Type DWord
    Set-ItemProperty -Path "$FirefoxPath\WebsiteFilter" -Name "Enabled" -Value 1 -Type DWord
    Set-ItemProperty -Path "$FirefoxPath\WebsiteFilter\Block" -Name "1" -Value "<all_urls>"
    Set-ItemProperty -Path "$FirefoxPath\WebsiteFilter\Exceptions" -Name "1" -Value "*://*.$AllowedSite/*"
    Set-ItemProperty -Path "$FirefoxPath\WebsiteFilter\Exceptions" -Name "2" -Value "*://$AllowedSite/*"

    # 3. Block alternative browsers for all standard profiles
    $UserProfiles = Get-CimInstance Win32_UserProfile | Where-Object { -not $_.Special }
    foreach ($Profile in $UserProfiles) {
        $SID = $Profile.SID
        $HiveLoaded = $false
        if (-not (Test-Path "Registry::HKEY_USERS\$SID")) {
            reg.exe load "HKU\$SID" "$($Profile.LocalPath)\NTUSER.DAT" | Out-Null
            $HiveLoaded = $true
        }

        $ExpPath = "Registry::HKEY_USERS\$SID\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer"
        $DisPath = "$ExpPath\DisallowRun"
        New-Item -Path $DisPath -Force | Out-Null
        Set-ItemProperty -Path $ExpPath -Name "DisallowRun" -Value 1 -Type DWord

        $i = 1
        foreach ($Exe in $BlockedExes) {
            Set-ItemProperty -Path $DisPath -Name "$i" -Value $Exe
            $i++
        }

        if ($HiveLoaded) {
            [gc]::Collect()
            Start-Sleep -Seconds 1
            reg.exe unload "HKU\$SID" | Out-Null
        }
    }

    Stop-Process -Name "chrome", "msedge", "brave", "firefox", "opera" -Force -ErrorAction SilentlyContinue
    Write-Host "`n[SUCCESS] Lockdown applied! Only https://$AllowedSite is allowed." -ForegroundColor Green

} elseif ($Choice -eq "2") {
    if (-not (Test-AdminPassword)) {
        Write-Host "`n[ACCESS DENIED] Incorrect password! Lockdown remains active." -ForegroundColor Red
        Read-Host "`nPress Enter to exit"
        exit
    }

    foreach ($Path in $ChromiumPaths) {
        Remove-Item -Path "$Path\URLBlocklist", "$Path\URLAllowlist" -Recurse -ErrorAction SilentlyContinue
        Remove-ItemProperty -Path $Path -Name "DownloadRestrictions" -ErrorAction SilentlyContinue
    }

    Remove-Item -Path "$FirefoxPath\WebsiteFilter" -Recurse -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path $FirefoxPath -Name "DisableDownloads" -ErrorAction SilentlyContinue

    $UserProfiles = Get-CimInstance Win32_UserProfile | Where-Object { -not $_.Special }
    foreach ($Profile in $UserProfiles) {
        $SID = $Profile.SID
        $HiveLoaded = $false
        if (-not (Test-Path "Registry::HKEY_USERS\$SID")) {
            reg.exe load "HKU\$SID" "$($Profile.LocalPath)\NTUSER.DAT" | Out-Null
            $HiveLoaded = $true
        }

        $ExpPath = "Registry::HKEY_USERS\$SID\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer"
        Remove-ItemProperty -Path $ExpPath -Name "DisallowRun" -ErrorAction SilentlyContinue
        Remove-Item -Path "$ExpPath\DisallowRun" -Recurse -ErrorAction SilentlyContinue

        if ($HiveLoaded) {
            [gc]::Collect()
            Start-Sleep -Seconds 1
            reg.exe unload "HKU\$SID" | Out-Null
        }
    }

    Write-Host "`n[SUCCESS] Password verified. Lockdown removed!" -ForegroundColor Yellow
}

Read-Host "`nPress Enter to exit"
