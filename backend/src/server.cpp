#include <algorithm>
#include <cstdlib>
#include <iostream>
#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <unistd.h>
#include <bitset>
#include <nlohmann/json.hpp>
#include <poll.h>
#include "openssl/sha.h"
#include "server.h"
#include "http.h"
#include "src/utils.h"
#include "trie/trie.h"
#include "websocket.h"
using namespace std;
using json = nlohmann::json;

#define WEBSOCKET_UUID_STRING "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"

Server::Server(char const *port, int max_buf_size, int backlog)
{
    this->port = port;
    this->backlog = backlog;
    this->max_buf_size = max_buf_size;
    this->router = new Trie("/");
}

void Server::run()
{
    int sockfd;
    addrinfo hints, *p, *serverinfo;
    sockaddr_storage client_addr;
    socklen_t client_addrlen = sizeof(client_addr);
    int yes = 1;

    std::vector<pollfd> pfds;
    // TODO: use a map instead
    std::vector<int> ws_fds;

    memset(&hints, 0, sizeof(hints));
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_flags = AI_PASSIVE;
    hints.ai_family = AF_INET;

    int status = getaddrinfo(nullptr, this->port, &hints, &serverinfo);

    if (status != 0) {
        perror("getaddrinfo error");
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

        if (::bind(sockfd, p->ai_addr, p->ai_addrlen) == -1) {
            continue;
        }

        break;
    }

    if (p == nullptr) {
        perror("binding error");
        exit(EXIT_FAILURE);
    }

    if (listen(sockfd, this->backlog) == -1) {
        perror("listening error");
        exit(EXIT_FAILURE);
    }

    freeaddrinfo(serverinfo);

    pollfd listener;
    listener.fd = sockfd;
    listener.events = POLLIN;
    pfds.push_back(listener);

    std::cout << "listening on port " << this->port << endl;
    while (true) {
        int poll_count = poll(pfds.data(), pfds.size(), -1);

        if (poll_count == -1) {
            perror("poll");
            exit(EXIT_FAILURE);
        }

        for (int i = 0; i < pfds.size(); i++) {
            pollfd &p = pfds[i];
            if (p.revents & POLLIN) {
                if (p.fd == sockfd) {
                    // listener is ready to accept new connection
                    int clientfd = accept(
                        sockfd, reinterpret_cast<sockaddr *>(&client_addr),
                        &client_addrlen);
                    if (clientfd == -1) {
                        perror("accept");
                    }
                    else {
                        pollfd newsock;
                        newsock.fd = clientfd;
                        newsock.events = POLLIN;
                        pfds.push_back(newsock);
                    }
                }
                else {
                    // a client socket has data to read
                    int clientfd = p.fd;

                    // handle websocket message
                    auto pos =
                        std::find(ws_fds.begin(), ws_fds.end(), clientfd) -
                        ws_fds.begin();

                    if (pos < ws_fds.size()) {
                        unsigned char buf[this->max_buf_size];
                        int received =
                            recv(clientfd, buf, this->max_buf_size, 0);

                        if (received == 0) {
                            ws_fds.erase(ws_fds.begin() + pos);
                            close(clientfd);
                            continue;
                        }

                        WebsocketFrame ws(buf);
                        // auto msg = ws.msg;
                        //
                        //
                        // auto frame = WebsocketFrame::createFrame(payload);
                        // do whatever with the json

                        // int bytes_sent =
                        //     send(clientfd, response_buf,
                        //          bytes_written + json_str.size(), 0);
                        // if (bytes_sent == -1) {
                        //     perror("byte sent error");
                        // }
                        //
                        continue;
                    }

                    // handle http requests

                    char buf[this->max_buf_size];

                    int bytes = recv(clientfd, buf, this->max_buf_size - 1, 0);
                    if (bytes == -1) {
                        perror("recv error");
                        continue;
                    }
                    else if (bytes == 0) {
                        // client disconnect
                        pfds.erase(pfds.begin() + i);
                        close(clientfd); // bye
                        continue;
                    }
                    buf[bytes] = '\0';

                    auto req = this->process_request(buf);
                    HTTP http(clientfd, req);

                    if (req.isWebsocketHandshake) {
                        std::cout << "Handshake" << endl;
                        string response = http.websocket_handshake();

                        int bytes =
                            send(clientfd, response.data(), response.size(), 0);

                        if (bytes == -1) {
                            perror("send error");
                            close(clientfd);
                        }

                        ws_fds.push_back(clientfd);
                    }
                    else {
                        std::cout << "route" << endl;
                        auto route = this->router->find(req.path);

                        if (route) {
                            auto route_handler = route->value;

                            if (route_handler) {
                                if (route->isWildcard) {
                                    req.param = route->wildcardContent;
                                }
                                route_handler(req, http);
                            }
                        }
                        else {
                            std::cout << "route not found: " << req.path
                                      << endl;
                            string response = http.not_found();
                            int bytes = send(clientfd, response.data(),
                                             response.size(), 0);

                            if (bytes == -1) {
                                perror("send error");
                            }
                        }
                    }
                }
            }
        }
    }

    close(sockfd);
}

http_request Server::process_request(char *buf)
{
    string http_msg(buf);
    http_request request;

    auto lines = utils::split_str(http_msg, "\r\n");
    auto tokens =
        utils::split_str(lines[0], " "); // lines[0] is the http startline

    request.method = tokens[0];
    request.path = tokens[1];
    request.param = "";

    for (int i = 1; i < lines.size(); i++) {
        auto tokens = utils::split_str(lines[i], ": ");
        request.headers[tokens[0]] = tokens[1];
    }

    if (request.headers["Upgrade"] == "websocket" &&
        request.headers["Connection"] == "Upgrade" &&
        request.headers.count("Sec-WebSocket-Key") > 0) {
        request.isWebsocketHandshake = true;
    }

    return request;
}

void Server::route(string path, Handler handler)
{
    this->router->insert(path, handler);
}
