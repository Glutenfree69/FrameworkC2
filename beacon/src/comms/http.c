/*
 * http.c - Custom HTTP implementation with AFD driver
 * Construction manuelle des requêtes HTTP/1.1 et parsing des réponses
 */

#include "http.h"
#include "afd.h"
#include "../types.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

int http_init(void) {
    // Rien à initialiser (pas de WinHTTP)
    return 1;
}

void http_cleanup(void) {
    // Rien à nettoyer
}

// Parser la réponse HTTP pour extraire status code et body
static void parse_http_response(const char* data, int data_len, HttpResponse* response) {
    const char* status_line;
    const char* body_start;
    int body_len;

    if (response == NULL || data == NULL || data_len <= 0) {
        return;
    }

    response->status_code = 0;
    response->body[0] = '\0';
    response->body_length = 0;

    // 1. Extraire le status code de la première ligne
    // Format: "HTTP/1.1 200 OK\r\n"
    status_line = data;
    if (data_len >= 12 && (strncmp(status_line, "HTTP/1.1 ", 9) == 0 ||
                           strncmp(status_line, "HTTP/1.0 ", 9) == 0)) {
        response->status_code = atoi(status_line + 9);
    }

    // 2. Trouver la séparation headers/body ("\r\n\r\n")
    body_start = strstr(data, "\r\n\r\n");
    if (body_start != NULL) {
        body_start += 4;  // Skip "\r\n\r\n"

        body_len = data_len - (int)(body_start - data);
        if (body_len > 0 && body_len < LARGE_BUF) {
            memcpy(response->body, body_start, body_len);
            response->body[body_len] = '\0';
            response->body_length = body_len;
        }
    }
}

// Fonction générique pour faire une requête HTTP
static int http_request(const char* method, const char* path, const char* body, HttpResponse* response) {
    HANDLE hSocket = NULL;
    char request[4096];
    char recv_buffer[LARGE_BUF];
    int request_len;
    int sent;
    int total_recv = 0;
    int bytes;

    // 1. Créer socket AFD
    hSocket = afd_socket_create();
    if (hSocket == NULL) {
        return 0;
    }

    // 2. Se connecter au serveur
    if (!afd_connect(hSocket, SERVER_IP, SERVER_PORT)) {
        afd_close(hSocket);
        return 0;
    }

    // 3. Construire la requête HTTP
    if (body != NULL) {
        // POST avec body
        request_len = snprintf(request, sizeof(request),
            "%s %s HTTP/1.1\r\n"
            "Host: %s:%d\r\n"
            "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36\r\n"
            "Content-Type: application/json\r\n"
            "Content-Length: %d\r\n"
            "Connection: close\r\n"
            "\r\n"
            "%s",
            method, path, SERVER_IP, SERVER_PORT, (int)strlen(body), body
        );
    } else {
        // GET sans body
        request_len = snprintf(request, sizeof(request),
            "%s %s HTTP/1.1\r\n"
            "Host: %s:%d\r\n"
            "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36\r\n"
            "Connection: close\r\n"
            "\r\n",
            method, path, SERVER_IP, SERVER_PORT
        );
    }

    // 4. Envoyer la requête
    sent = afd_send(hSocket, request, request_len);
    if (sent <= 0) {
        afd_close(hSocket);
        return 0;
    }

    // 5. Recevoir la réponse
    memset(recv_buffer, 0, sizeof(recv_buffer));
    total_recv = 0;

    while (total_recv < LARGE_BUF - 1) {
        bytes = afd_recv(hSocket, recv_buffer + total_recv, LARGE_BUF - 1 - total_recv);
        if (bytes <= 0) {
            break;  // Connexion fermée ou erreur
        }
        total_recv += bytes;
    }

    recv_buffer[total_recv] = '\0';

    // 6. Parser la réponse HTTP
    if (response != NULL) {
        parse_http_response(recv_buffer, total_recv, response);
    }

    // 7. Fermer le socket
    afd_close(hSocket);

    // Success si on a reçu une réponse 2xx
    return (response && response->status_code >= 200 && response->status_code < 300) ? 1 : 0;
}

// Wrappers publics
int http_get(const char* path, HttpResponse* response) {
    return http_request("GET", path, NULL, response);
}

int http_post(const char* path, const char* json_body, HttpResponse* response) {
    return http_request("POST", path, json_body, response);
}
