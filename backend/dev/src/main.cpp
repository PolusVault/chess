#include "server.h"
#include "src/http.h"
#include <iostream>

#define PORT "9034"
#define BACKLOG 10
#define MAX_BUF_SIZE 4096

void root(http_request &req, HTTP &http)
{
    http.sendFile("../dist/index.html");
}

void root2(http_request &req, HTTP &http)
{
    http.sendFile("../dist/" + req.param);
}

void assets(http_request &req, HTTP &http)
{
    http.sendFile("../dist/assets/" + req.param);
}

int main()
{
    Server server(PORT, MAX_BUF_SIZE, BACKLOG);
    server.route("/", &root);
    server.route("/*", &root2);
    server.route("/assets/*", &assets);

    server.run();
}
