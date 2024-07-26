#pragma once
#include "http.h"
#include "trie/trie.h"
#include "utils.h"

class Server {
    char const *port;
    int backlog;
    int max_buf_size;
    Trie *router;

    http_request process_request(char *buf);
    void handle_http(int fd);
    void handle_websocket(int fd);

  public:
    Server(char const *port, int max_buf_size, int backlog = 10);
    void run();
    void route(string path, Handler handler);
};
