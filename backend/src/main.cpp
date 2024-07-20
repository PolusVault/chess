#include "server.h"
#include "trie/trie.h"
using namespace std;

#define PORT "9034"
#define BACKLOG 10
#define MAX_BUF_SIZE 4096

int main()
{
    Trie a("/");

    a.insert("/about");
    a.insert("/product");
    a.insert("/product/entertainment");
    a.insert("/product/entertainment/laptop");

    a.display();
    // Server server(PORT, MAX_BUF_SIZE, BACKLOG);
    //
    // server.route("/", 0);
    //
    // server.run();
}
