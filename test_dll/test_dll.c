#include <windows.h>

// Thread qui affiche la MessageBox
DWORD WINAPI MessageBoxThread(LPVOID lpParam) {
    MessageBoxA(
        NULL,
        "JE PEUX FAIRE CE QUE JE VEUX UWU",
        "Oh zebi une DLL",
        MB_OK | MB_ICONINFORMATION
    );
    return 0;
}

// DllMain - crée un thread au chargement
BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
    switch (fdwReason) {
        case DLL_PROCESS_ATTACH:
            // Désactiver les notifications de thread pour optimiser
            DisableThreadLibraryCalls(hinstDLL);
            
            // Créer un thread pour afficher la MessageBox
            CreateThread(NULL, 0, MessageBoxThread, NULL, 0, NULL);
            break;
        case DLL_PROCESS_DETACH:
            break;
    }
    return TRUE;
}
