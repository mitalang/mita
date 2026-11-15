#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#define BUF_SIZE 65536

static char g_buffer[BUF_SIZE];
static int g_buffer_len = 0;

int64_t mita_net_listen(int64_t port) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return -1;

    int opt = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons((int)port);

    if (bind(fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        close(fd);
        return -2;
    }

    if (listen(fd, 10) < 0) {
        close(fd);
        return -3;
    }

    return fd;
}

int64_t mita_net_accept(int64_t listen_fd) {
    struct sockaddr_in client_addr;
    socklen_t addr_len = sizeof(client_addr);
    int client_fd = accept((int)listen_fd, (struct sockaddr*)&client_addr, &addr_len);
    return client_fd;
}

int64_t mita_net_read(int64_t fd) {
    g_buffer_len = recv((int)fd, g_buffer, BUF_SIZE - 1, 0);
    if (g_buffer_len > 0) {
        g_buffer[g_buffer_len] = '\0';
    }
    return g_buffer_len;
}

const char* mita_net_get_buffer(void) {
    return g_buffer;
}

int64_t mita_net_write(int64_t fd, const char* data) {
    return send((int)fd, data, strlen(data), 0);
}

int64_t mita_net_close(int64_t fd) {
    return close((int)fd);
}

const char* mita_http_parse_path(const char* request) {
    static char path[256];
    path[0] = '\0';
    
    if (strncmp(request, "GET ", 4) == 0 || strncmp(request, "POST ", 5) == 0) {
        const char* start = strchr(request, ' ') + 1;
        const char* end = strchr(start, ' ');
        if (end) {
            size_t len = end - start;
            if (len < sizeof(path)) {
                strncpy(path, start, len);
                path[len] = '\0';
            }
        }
    }
    return path;
}

const char* mita_http_ok_response(const char* body) {
    static char response[BUF_SIZE];
    int body_len = strlen(body);
    snprintf(response, sizeof(response),
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: text/html; charset=utf-8\r\n"
        "Content-Length: %d\r\n"
        "Connection: close\r\n"
        "\r\n"
        "%s", body_len, body);
    return response;
}

const char* mita_http_not_found_response(const char* body) {
    static char response[BUF_SIZE];
    int body_len = strlen(body);
    snprintf(response, sizeof(response),
        "HTTP/1.1 404 Not Found\r\n"
        "Content-Type: text/html; charset=utf-8\r\n"
        "Content-Length: %d\r\n"
        "Connection: close\r\n"
        "\r\n"
        "%s", body_len, body);
    return response;
}
