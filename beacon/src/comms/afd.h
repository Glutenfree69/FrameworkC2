#ifndef AFD_H
#define AFD_H

#include <windows.h>

// AFD socket functions (syscall-based)
HANDLE afd_socket_create(void);
int afd_connect(HANDLE hAfd, const char* ip, unsigned short port);
int afd_send(HANDLE hAfd, const char* data, int len);
int afd_recv(HANDLE hAfd, char* buffer, int buflen);
void afd_close(HANDLE hAfd);

// Helper functions
unsigned long custom_inet_addr(const char* ip);
unsigned short custom_htons(unsigned short hostshort);

#endif // AFD_H
