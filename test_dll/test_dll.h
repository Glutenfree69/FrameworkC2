#ifndef TEST_DLL_H
#define TEST_DLL_H

#ifdef __cplusplus
extern "C" {
#endif

__declspec(dllexport) void ShowMessage(void);

#ifdef __cplusplus
}
#endif

#endif // TEST_DLL_H
