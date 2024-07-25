#include "server.h"
#include "src/http.h"
#include "trie/trie.h"
#include "nlohmann/json.hpp"
#include <_types/_uint8_t.h>
#include <algorithm>
#include <iostream>
#include <sys/_endian.h>
using namespace std;

using json = nlohmann::json;
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

union uint16_t_converter {
    uint16_t i;
    uint8_t c[2];
};

int main()
{
    Server server(PORT, MAX_BUF_SIZE, BACKLOG);
    server.route("/", &root);
    server.route("/*", &root2);
    server.route("/assets/*", &assets);

    server.run();
}
