#include <windows.h>
#include <shellapi.h>
#include <bcrypt.h>
#include <conio.h>
#include <iostream>
#include <sstream>
#include <iomanip>
#include <string>
#include <vector>
#include <cstdlib>

// Link Windows libraries (for MSVC; for g++, use -lbcrypt -lshell32 -ladvapi32)
#pragma comment(lib, "bcrypt.lib")
#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "advapi32.lib")

void runCmd(const std::string& cmd) {
    std::system((cmd + " >nul 2>&1").c_str());
}

// Check if currently running with Administrator privileges
bool isRunningAsAdmin() {
    BOOL isAdmin = FALSE;
    PSID adminGroup = NULL;
    SID_IDENTIFIER_AUTHORITY ntAuthority = SECURITY_NT_AUTHORITY;
    if (AllocateAndInitializeSid(&ntAuthority, 2, SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS, 0, 0, 0, 0, 0, 0, &adminGroup)) {
        CheckTokenMembership(NULL, adminGroup, &isAdmin);
        FreeSid(adminGroup);
    }
    return isAdmin != FALSE;
}

// Compute uppercase SHA-256 hex string using Windows CNG API
std::string sha256(const std::string& input) {
    BCRYPT_ALG_HANDLE hAlg = NULL;
    BCRYPT_HASH_HANDLE hHash = NULL;
    BYTE hash[32];
    DWORD cbData = 0, cbHashObject = 0;

    BCryptOpenAlgorithmProvider(&hAlg, BCRYPT_SHA256_ALGORITHM, NULL, 0);
    BCryptGetProperty(hAlg, BCRYPT_OBJECT_LENGTH, (PBYTE)&cbHashObject, sizeof(DWORD), &cbData, 0);
    std::vector<BYTE> hashObject(cbHashObject);

    BCryptCreateHash(hAlg, &hHash, hashObject.data(), cbHashObject, NULL, 0, 0);
    BCryptHashData(hHash, (PBYTE)input.c_str(), (ULONG)input.length(), 0);
    BCryptFinishHash(hHash, hash, 32, 0);

    BCryptDestroyHash(hHash);
    BCryptCloseAlgorithmProvider(hAlg, 0);

    std::ostringstream oss;
    for (int i = 0; i < 32; ++i) {
        oss << std::uppercase << std::hex << std::setw(2) << std::setfill('0') << (int)hash[i];
    }
    return oss.str();
}

// Read password from console while masking characters with '*'
std::string getMaskedPassword() {
    std::string pass;
    char ch;
    while ((ch = _getch()) != '\r') {
        if (ch == '\b') {
            if (!pass.empty()) {
                pass.pop_back();
                std::cout << "\b \b";
            }
        } else if (ch >= 32 && ch <= 126) {
            pass.push_back(ch);
            std::cout << '*';
        }
    }
    std::cout << "\n";
    return pass;
}

int main() {
    // 1. Auto-elevate to Administrator if not already running as Admin
    if (!isRunningAsAdmin()) {
        char szPath[MAX_PATH];
        if (GetModuleFileNameA(NULL, szPath, MAX_PATH)) {
            ShellExecuteA(NULL, "runas", szPath, NULL, NULL, SW_SHOWNORMAL);
        }
        return 0;
    }

    // Target domain (browser policy syntax uses domain without https://)
    const std::string site = "bapsoj.org";

    // Default SHA-256 hash below is for password: MySecretPass123!
    const std::string STORED_HASH = "99A4E22854E630750D579F4C987DC57795EC6219DF8EE7939BC8BF5F61431B32";

    int choice = 0;
    std::cout << "==========================================\n";
    std::cout << "   CONTEST LOCKDOWN TOOL (" << site << ")\n";
    std::cout << "==========================================\n";
    std::cout << "1. ENABLE Lockdown (Allow ONLY https://" << site << ")\n";
    std::cout << "2. DISABLE Lockdown (Password Protected)\n";
    std::cout << "Select (1 or 2): ";
    std::cin >> choice;

    std::vector<std::string> chromiumPaths = {
        "HKLM\\SOFTWARE\\Policies\\Google\\Chrome",
        "HKLM\\SOFTWARE\\Policies\\Microsoft\\Edge",
        "HKLM\\SOFTWARE\\Policies\\BraveSoftware\\Brave"
    };

    if (choice == 1) {
        // Lock down Chrome, Edge, and Brave
        for (const auto& path : chromiumPaths) {
            runCmd("reg delete \"" + path + "\\URLBlocklist\" /f");
            runCmd("reg delete \"" + path + "\\URLAllowlist\" /f");
            runCmd("reg add \"" + path + "\\URLBlocklist\" /v \"1\" /t REG_SZ /d \"*\" /f");
            runCmd("reg add \"" + path + "\\URLAllowlist\" /v \"1\" /t REG_SZ /d \"" + site + "\" /f");
            runCmd("reg add \"" + path + "\\URLAllowlist\" /v \"2\" /t REG_SZ /d \"*." + site + "\" /f");
            // DownloadRestrictions: 1 = Block dangerous/exe downloads, 3 = Block ALL
            runCmd("reg add \"" + path + "\" /v \"DownloadRestrictions\" /t REG_DWORD /d 1 /f");
        }

        // Lock down Mozilla Firefox
        std::string ff = "HKLM\\SOFTWARE\\Policies\\Mozilla\\Firefox";
        runCmd("reg delete \"" + ff + "\\WebsiteFilter\" /f");
        runCmd("reg add \"" + ff + "\" /v \"DisableDownloads\" /t REG_DWORD /d 1 /f");
        runCmd("reg add \"" + ff + "\\WebsiteFilter\" /v \"Enabled\" /t REG_DWORD /d 1 /f");
        runCmd("reg add \"" + ff + "\\WebsiteFilter\\Block\" /v \"1\" /t REG_SZ /d \"<all_urls>\" /f");
        runCmd("reg add \"" + ff + "\\WebsiteFilter\\Exceptions\" /v \"1\" /t REG_SZ /d \"*://*." + site + "/*\" /f");
        runCmd("reg add \"" + ff + "\\WebsiteFilter\\Exceptions\" /v \"2\" /t REG_SZ /d \"*://" + site + "/*\" /f");

        // Block alternative & portable browsers via DisallowRun
        std::string exp = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer";
        runCmd("reg add \"" + exp + "\" /v \"DisallowRun\" /t REG_DWORD /d 1 /f");
        std::vector<std::string> exes = {
            "opera.exe", "vivaldi.exe", "tor.exe", "waterfox.exe", "maxthon.exe",
            "IronPortable.exe", "FirefoxPortable.exe", "GoogleChromePortable.exe",
            "OperaPortable.exe", "BravePortable.exe"
        };
        for (size_t i = 0; i < exes.size(); ++i) {
            runCmd("reg add \"" + exp + "\\DisallowRun\" /v \"" + std::to_string(i + 1) + "\" /t REG_SZ /d \"" + exes[i] + "\" /f");
        }

        // Force-close browsers so policies take effect immediately
        runCmd("taskkill /F /IM chrome.exe /IM msedge.exe /IM brave.exe /IM firefox.exe /IM opera.exe");
        std::cout << "\n[SUCCESS] Lockdown active! Only https://" << site << " is accessible.\n";
    }
    else if (choice == 2) {
        std::cout << "Enter Admin Password: ";
        std::string inputPass = getMaskedPassword();

        if (sha256(inputPass) != STORED_HASH) {
            std::cout << "\n[ACCESS DENIED] Invalid password! Lockdown remains active.\n";
            std::system("pause");
            return 1;
        }

        for (const auto& path : chromiumPaths) {
            runCmd("reg delete \"" + path + "\\URLBlocklist\" /f");
            runCmd("reg delete \"" + path + "\\URLAllowlist\" /f");
            runCmd("reg delete \"" + path + "\" /v \"DownloadRestrictions\" /f");
        }

        runCmd("reg delete \"HKLM\\SOFTWARE\\Policies\\Mozilla\\Firefox\\WebsiteFilter\" /f");
        runCmd("reg delete \"HKLM\\SOFTWARE\\Policies\\Mozilla\\Firefox\" /v \"DisableDownloads\" /f");
        runCmd("reg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer\" /v \"DisallowRun\" /f");
        runCmd("reg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\Explorer\\DisallowRun\" /f");

        std::cout << "\n[SUCCESS] Password verified. All restrictions removed!\n";
    }

    std::system("pause");
    return 0;
}
