// #include "server.h"
#include "trie/trie.h"
using namespace std;

#define PORT "9034"
#define BACKLOG 10
#define MAX_BUF_SIZE 4096

void root(http_request &req, HTTP &http)
{
    http.sendFile("../dist/index.html");
}

int main()
{
    Trie trie("/");

    trie.insert("/", &root);
    // trie.insert("/home", whatever_function_goes_here);
}
