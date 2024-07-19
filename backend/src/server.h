#pragma once
#include "http.h"

class Server {
    char const *port;
    int backlog;
    int max_buf_size;
    HTTP* http;

  public:
    Server(char const *port, int max_buf_size, int backlog = 10);
    void run();
};
