#pragma once
#include <poll.h>
#include <set>
#include "http.h"
#include "trie/trie.h"
#include "utils.h"
#include "websocket.h"

struct Connection {
    bool __is_listener;

    string ip_addr;
    int fd;

    bool is_websocket;
    bool is_dirty;

    void mark_dirty()
    {
        assert(__is_listener == false && "can't modify the listener socket");
        is_dirty = true;
    }
};

class Server {
    char const *port;
    int backlog;
    int max_buf_size;
    Trie *router;

    int listenerfd;

    std::vector<pollfd> pfds;
    std::vector<Connection> connections;

    http_request process_request(char *buf);
    void handle_new_conn();
    void handle_incoming(Connection &conn);
    void handle_http(Connection &conn);
    void handle_websocket(Connection &conn);
    void cleanup();

    ssize_t send(int, const void *, size_t, int = 0);
    ssize_t recv(int, void *, size_t, int = 0);

  public:
    Server(char const *port, int max_buf_size, int backlog = 10);
    void run();
    void route(string path, RouteHandler handler);
};
