/*******************************************************************************
*
*  UAC Bypass via ICMLuaUtil COM Interface
*  
*  Educational implementation based on UACME project (hfiref0x)
*  https://github.com/hfiref0x/UACME - Method 41 (Oddvar Moe)
*
*  This code demonstrates UAC bypass using elevated COM interface.
*  For educational purposes only - study how Windows COM elevation works.
*
*******************************************************************************/
#ifndef UAC_BYPASS_H
#define UAC_BYPASS_H

#include <windows.h>

#ifdef __cplusplus
extern "C" {
#endif

// Result codes for UAC bypass operations
typedef enum _UAC_RESULT {
    UAC_SUCCESS = 0,
    UAC_ERROR_COM_INIT = 1,
    UAC_ERROR_ELEVATE_OBJECT = 2,
    UAC_ERROR_SHELL_EXEC = 3,
    UAC_ERROR_INVALID_PARAM = 4,
} UAC_RESULT;

/**
 * @brief Execute a command with elevated privileges using ICMLuaUtil COM interface
 * 
 * This function uses the ICMLuaUtil COM interface which is auto-elevated
 * by Windows. It allows executing commands without triggering UAC prompt.
 * 
 * The technique works because:
 * 1. CMSTPLUA COM object is in the UAC auto-approval list
 * 2. ICMLuaUtil interface has a ShellExec method that runs elevated
 * 3. Windows trusts this interface for Connection Manager operations
 * 
 * @param lpFile         Path to executable to run elevated
 * @param lpParameters   Command line parameters (can be NULL)
 * @param lpDirectory    Working directory (can be NULL)
 * @param nShow          Window show state (SW_HIDE, SW_SHOW, etc.)
 * @return UAC_RESULT    UAC_SUCCESS on success, error code otherwise
 */
UAC_RESULT UacBypassShellExec(
    _In_     LPCWSTR lpFile,
    _In_opt_ LPCWSTR lpParameters,
    _In_opt_ LPCWSTR lpDirectory,
    _In_     int nShow
);

/**
 * @brief Check if current process could potentially use UAC bypass
 * 
 * Returns TRUE if:
 * - Running as a member of Administrators group
 * - Process is NOT already elevated
 * - UAC is likely enabled (not 100% accurate check)
 * 
 * @return BOOL TRUE if bypass might work, FALSE otherwise
 */
BOOL UacBypassCanAttempt(void);

#ifdef __cplusplus
}
#endif

#endif // UAC_BYPASS_H
