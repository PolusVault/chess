#include "server.h"
using namespace std;

#define PORT "9034"
#define BACKLOG 10
#define MAX_BUF_SIZE 4096

int main()
{
    Server server(PORT, MAX_BUF_SIZE, BACKLOG);
    server.run();
}
