#include <algorithm>
#include <cstdlib>
#include <iostream>
#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <unistd.h>
#include <poll.h>
#include <arpa/inet.h>
#include "openssl/sha.h"
#include "server.h"
#include "http.h"
#include "src/utils.h"
#include "trie/trie.h"
#include "websocket.h"
#include "spdlog/spdlog.h"

#include <nlohmann/json.hpp>
using json = nlohmann::json;

Server::Server(char const *port, int max_buf_size, int backlog)
{
    this->port = port;
    this->backlog = backlog;
    this->max_buf_size = max_buf_size;
    this->router = new Trie("/");
}

void Server::run()
{
    addrinfo hints, *p, *serverinfo;
    int yes = 1;

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
        this->listenerfd = socket(p->ai_family, p->ai_socktype, p->ai_protocol);
        if (this->listenerfd == -1) {
            continue;
        }

        // avoid the binding error "address already in use"
        if (setsockopt(listenerfd, SOL_SOCKET, SO_REUSEADDR, &yes,
                       sizeof(int)) == -1) {
            perror("setsockopt");
            exit(1);
        }

        if (bind(this->listenerfd, p->ai_addr, p->ai_addrlen) == -1) {
            continue;
        }

        break;
    }

    if (p == nullptr) {
        perror("binding error");
        exit(EXIT_FAILURE);
    }

    if (listen(this->listenerfd, this->backlog) == -1) {
        perror("listening error");
        exit(EXIT_FAILURE);
    }

    freeaddrinfo(serverinfo);

    pollfd listener = {.fd = this->listenerfd, .events = POLLIN};
    this->connections.push_back(
        Connection{.__is_listener = true, .fd = this->listenerfd});
    this->pfds.push_back(listener);

    std::cout << "listening on port " << this->port << std::endl;

    while (true) {
        int poll_count = poll(this->pfds.data(), this->pfds.size(), -1);

        if (poll_count == -1) {
            perror("poll");
            exit(EXIT_FAILURE);
        }

        for (size_t i = 0; i < this->pfds.size(); i++) {
            auto &p = this->pfds[i];
            auto &conn = this->connections[i];
            assert(p.fd == conn.fd && "connections and pfds must be in sync");

            if (p.revents & POLLIN) {
                if (p.fd == this->listenerfd) {
                    spdlog::info("new connection");
                    this->handle_new_conn();
                }
                else {
                    spdlog::info("existing connection");
                    this->handle_incoming(conn);
                }
            }
        }

        this->cleanup();
    }

    close(this->listenerfd);
}

void Server::handle_new_conn()
{
    sockaddr_storage client_addr;
    socklen_t client_addrlen = sizeof(client_addr);
    char ip_addr[INET_ADDRSTRLEN];

    int clientfd =
        accept(this->listenerfd, reinterpret_cast<sockaddr *>(&client_addr),
               &client_addrlen);

    auto temp = reinterpret_cast<sockaddr *>(&client_addr);
    auto sin_addr = reinterpret_cast<sockaddr_in *>(temp)->sin_addr;

    inet_ntop(client_addr.ss_family, &sin_addr, ip_addr, sizeof(ip_addr));

    spdlog::info("IP Address: {}", ip_addr);

    if (clientfd == -1) {
        perror("accept");
    }
    else {
        pollfd p = {.fd = clientfd, .events = POLLIN};

        this->connections.push_back(Connection{
            .fd = clientfd,
            .ip_addr = ip_addr,
            .__is_listener = false,
            .is_websocket = false,
            .is_dirty = false,
        });

        this->pfds.push_back(p);
    }
}

void Server::handle_incoming(Connection &conn)
{
    if (conn.is_websocket) {
        this->handle_websocket(conn);
    }
    else {
        this->handle_http(conn);
    }
}

void Server::handle_http(Connection &conn)
{
    int fd = conn.fd;
    char buf[this->max_buf_size];

    int bytes_received = recv(fd, buf, this->max_buf_size - 1, 0);
    if (bytes_received == -1) {
        return;
    }
    else if (bytes_received == 0) {
        conn.mark_dirty();
        return;
    }
    buf[bytes_received] = '\0';

    auto req = this->process_request(buf);
    HTTP http(fd, req);

    if (req.isWebsocketHandshake) {
        string response = http.websocket_handshake();

        if (send(fd, response.data(), response.size(), 0) == -1) {
            conn.mark_dirty();
            return;
        }

        conn.is_websocket = true;
    }
    else {
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
            string response = http.not_found();

            if (send(fd, response.data(), response.size(), 0) == -1) {
                conn.mark_dirty();
            }
        }
    }
}

void Server::handle_websocket(Connection &conn)
{
    int fd = conn.fd;
    unsigned char buf[this->max_buf_size];
    int bytes_received = recv(fd, buf, this->max_buf_size, 0);

    if (bytes_received == -1) {
        return;
    }
    if (bytes_received == 0) {
        conn.mark_dirty();
        return;
    }

    auto data = ws::parse_frame(buf);

    if (data.is_close_frame) {
        spdlog::info("client disconnect");
        // client is disconnecting
        // send back a close frame in response
        auto buf = ws::create_close_frame();
        send(fd, buf, 2, 0);

        conn.mark_dirty();
        return;
    }
    else {
        spdlog::info("client sending data");
    }
}

void Server::cleanup()
{
    for (size_t i = 0; i < this->connections.size(); i++) {
        auto &conn = this->connections[i];
        auto &p = this->pfds[i];

        assert(p.fd == conn.fd && "connections and pfds must be in sync");

        if (conn.__is_listener)
            continue;

        if (conn.is_dirty) {
            close(conn.fd);
            this->connections.erase(this->connections.begin() + i);
            this->pfds.erase(this->pfds.begin() + i);
        }
    }

    spdlog::info("cleanup: {}", this->pfds.size());
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

void Server::route(string path, RouteHandler handler)
{
    this->router->insert(path, handler);
}

ssize_t Server::send(int fd, const void *buf, size_t buf_len, int flag)
{
    auto bytes_sent = ::send(fd, buf, buf_len, flag);

    if (bytes_sent == -1) {
        // TODO: log error here
        perror("sent error");
    }

    return bytes_sent;
}

ssize_t Server::recv(int fd, void *buf, size_t buf_len, int flag)
{
    auto bytes_received = ::recv(fd, buf, buf_len - 1, flag);

    if (bytes_received == -1) {
        perror("recv error");
    }

    return bytes_received;
}
