#include <cstdlib>
#include <iostream>
#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <unistd.h>
#include "server.h"
#include "http.h"
using namespace std;

Server::Server(char const *port, int max_buf_size, int backlog)
{
    this->port = port;
    this->backlog = backlog;
    this->max_buf_size = max_buf_size;
    this->http = new HTTP();
}

void Server::run() {
    int sockfd;
    addrinfo hints, *p, *serverinfo;
    sockaddr_storage client_addr;
    socklen_t client_addrlen = sizeof(client_addr);
    int yes = 1;

    memset(&hints, 0, sizeof(hints));
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_flags = AI_PASSIVE;
    hints.ai_family = AF_INET;

    int status = getaddrinfo(nullptr, this->port, &hints, &serverinfo);

    if (status != 0) {
        std::cout << "getaddrinfo error" << endl;
        exit(EXIT_FAILURE);
    }

    // bind to the first socket that works
    for (p = serverinfo; p != nullptr; p = p->ai_next) {
        sockfd = socket(p->ai_family, p->ai_socktype, p->ai_protocol);
        if (sockfd == -1) {
            continue;
        }

        // avoid the binding error "address already in use"
        if (setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(int)) ==
            -1) {
            perror("setsockopt");
            exit(1);
        }

        if (bind(sockfd, p->ai_addr, p->ai_addrlen) == -1) {
            continue;
        }

        break;
    }

    if (p == nullptr) {
        perror("binding error");
        exit(EXIT_FAILURE);
    }

    if (listen(sockfd, this->backlog) == -1) {
        std::cout << "listening error" << endl;
        exit(EXIT_FAILURE);
    }

    freeaddrinfo(serverinfo);

    std::cout << "listening on port " << this->port << endl;
    while (true) {
        int newfd = accept(sockfd, reinterpret_cast<sockaddr *>(&client_addr),
                           &client_addrlen);
        if (newfd == -1) {
            std::cout << "unable to create socket for client" << endl;
            continue;
        }

        std::cout << "got a connection" << endl;
        char buf[this->max_buf_size];

        int bytes = recv(newfd, buf, this->max_buf_size-1, 0);
        if (bytes == -1) {
            perror("recv error");
            continue;
        }
        buf[bytes] = '\0';
        this->http->process_request(buf, newfd);
        // this->http->handle_request(request);

        close(newfd);
    }

    close(sockfd);
}
