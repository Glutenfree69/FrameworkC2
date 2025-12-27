/*
 * http.h - Communication HTTP avec WinHTTP
 */

#ifndef HTTP_H
#define HTTP_H

#include "types.h"

int http_init(void);
void http_cleanup(void);
int http_get(const char* path, HttpResponse* response);
int http_post(const char* path, const char* json_body, HttpResponse* response);

#endif