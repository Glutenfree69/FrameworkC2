/*
 * http.c - Communication HTTP avec WinHTTP
 */

#include <windows.h>
#include <winhttp.h>
#include <stdio.h>
#include <string.h>
#include "http.h"

static HINTERNET g_hSession = NULL;

#define USER_AGENT L"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"

int http_init(void) {
    if (g_hSession != NULL) {
        return 1;
    }

    g_hSession = WinHttpOpen(
        USER_AGENT,
        WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
        WINHTTP_NO_PROXY_NAME,
        WINHTTP_NO_PROXY_BYPASS,
        0
    );

    if (g_hSession == NULL) {
        return 0;
    }

    return 1;
}

void http_cleanup(void) {
    if (g_hSession != NULL) {
        WinHttpCloseHandle(g_hSession);
        g_hSession = NULL;
    }
}

static int http_request(
    const char* method,
    const char* path,
    const char* body,
    HttpResponse* response
) {
    HINTERNET hConnect = NULL;
    HINTERNET hRequest = NULL;
    BOOL result = FALSE;
    DWORD bytesRead = 0;
    DWORD statusCode = 0;
    DWORD statusCodeSize = sizeof(statusCode);

    if (response != NULL) {
        response->status_code = 0;
        response->body[0] = '\0';
        response->body_length = 0;
    }

    if (g_hSession == NULL) {
        if (!http_init()) {
            return 0;
        }
    }

    wchar_t wPath[512];
    MultiByteToWideChar(CP_UTF8, 0, path, -1, wPath, 512);

    wchar_t wMethod[16];
    MultiByteToWideChar(CP_UTF8, 0, method, -1, wMethod, 16);

    wchar_t wServer[64];
    MultiByteToWideChar(CP_UTF8, 0, SERVER_IP, -1, wServer, 64);

    hConnect = WinHttpConnect(
        g_hSession,
        wServer,
        SERVER_PORT,
        0
    );

    if (hConnect == NULL) {
        goto cleanup;
    }

    hRequest = WinHttpOpenRequest(
        hConnect,
        wMethod,
        wPath,
        NULL,
        WINHTTP_NO_REFERER,
        WINHTTP_DEFAULT_ACCEPT_TYPES,
        0
    );

    if (hRequest == NULL) {
        goto cleanup;
    }

    if (body != NULL) {
        WinHttpAddRequestHeaders(
            hRequest,
            L"Content-Type: application/json",
            -1L,
            WINHTTP_ADDREQ_FLAG_ADD
        );
    }

    DWORD bodyLen = (body != NULL) ? (DWORD)strlen(body) : 0;
    result = WinHttpSendRequest(
        hRequest,
        WINHTTP_NO_ADDITIONAL_HEADERS,
        0,
        (LPVOID)body,
        bodyLen,
        bodyLen,
        0
    );

    if (!result) {
        goto cleanup;
    }

    result = WinHttpReceiveResponse(hRequest, NULL);
    if (!result) {
        goto cleanup;
    }

    WinHttpQueryHeaders(
        hRequest,
        WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
        WINHTTP_HEADER_NAME_BY_INDEX,
        &statusCode,
        &statusCodeSize,
        WINHTTP_NO_HEADER_INDEX
    );

    if (response != NULL) {
        response->status_code = (int)statusCode;
    }

    if (response != NULL) {
        DWORD totalRead = 0;

        do {
            DWORD available = 0;

            if (!WinHttpQueryDataAvailable(hRequest, &available)) {
                break;
            }

            if (available == 0) {
                break;
            }

            if (totalRead + available >= LARGE_BUF - 1) {
                available = LARGE_BUF - 1 - totalRead;
            }

            if (!WinHttpReadData(hRequest,
                                 response->body + totalRead,
                                 available,
                                 &bytesRead)) {
                break;
            }

            totalRead += bytesRead;

        } while (bytesRead > 0 && totalRead < LARGE_BUF - 1);

        response->body[totalRead] = '\0';
        response->body_length = totalRead;
    }

    result = TRUE;

cleanup:
    if (hRequest) WinHttpCloseHandle(hRequest);
    if (hConnect) WinHttpCloseHandle(hConnect);

    return (result && statusCode >= 200 && statusCode < 300) ? 1 : 0;
}

int http_get(const char* path, HttpResponse* response) {
    return http_request("GET", path, NULL, response);
}

int http_post(const char* path, const char* json_body, HttpResponse* response) {
    return http_request("POST", path, json_body, response);
}