#include <algorithm>
#include <cstdlib>
#include <iostream>
#include <sys/types.h>
#include <sys/socket.h>
#include <netdb.h>
#include <unistd.h>
#include <bitset>
#include <nlohmann/json.hpp>
#include "openssl/sha.h"
#include "server.h"
#include "http.h"
#include "src/utils.h"
#include "trie/trie.h"
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

union uint16_t_converter {
    uint16_t i;
    uint8_t c[2];
};

union uint64_t_converter {
    uint64_t i;
    uint8_t c[8];
};

// use https://github.com/zaphoyd/websocketpp/tree/master for reference
class WebsocketFrame {
    unsigned char mask;
    uint64_t length;

  public:
    string payload;

    WebsocketFrame(unsigned char *buf, int size)
    {
        if (size <= 0)
            return;

        auto fin_and_opcode = buf[0];
        auto mask_and_length = buf[1];

        // clang-format off
        this->mask   = mask_and_length & 0b10000000; // mask is in the first bit
        this->length = mask_and_length & 0b01111111; // length is the rest of the bits after the first
        // clang-format on

        int mask_offset = 2;

        if (this->length == 126) {
            uint16_t_converter temp;
            std::copy(buf + 2, buf + 5, temp.c);
            this->length = ntohs(temp.i);
            mask_offset = 4;
        }
        else if (this->length == 127) {
            // NOTE: how do we test this?
            uint64_t_converter temp;
            std::copy(buf + 2, buf + 10, temp.c);
            // TODO: convert this to host byte order too
            this->length = temp.i;
            mask_offset = 10;
        }

        // char msgbuf[this->length];
        unsigned char mask[4] = {buf[mask_offset], buf[mask_offset + 1],
                                 buf[mask_offset + 2], buf[mask_offset + 3]};
        int payload_offset = mask_offset + 4;

        // for (int i = 0; i < this->length; i++) {
        //     // unmask the payload
        //     msgbuf[i] = buf[payload_offset + i] ^ mask[i % 4];
        // }
        // msgbuf[this->length] = '\0';

        std::cout << "LENGTH: " << this->length << endl;
        // std::cout << "MESSAGE: " << msgbuf << endl;
        //
        // json temp = json::parse(msgbuf, msgbuf + this->length);
        // std::cout << temp["id"] << endl;

        // TODO: create a message object from the json to use
        // see notes.md for formats
    };
};

void Server::run()
{
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

        int bytes = recv(newfd, buf, this->max_buf_size - 1, 0);
        if (bytes == -1) {
            perror("recv error");
            continue;
        }
        buf[bytes] = '\0';

        auto req = this->process_request(buf);

        if (req.isWebsocketHandshake) {
            string key =
                req.headers["Sec-WebSocket-Key"] + WEBSOCKET_UUID_STRING;
            // convert string to unsigned char
            std::vector<unsigned char> vec(key.begin(), key.end());
            const unsigned char *str = vec.data();

            unsigned char hash[SHA_DIGEST_LENGTH]; // == 20

            SHA1(str, vec.size(), hash);

            string base64_key = utils::base64_encode(hash, SHA_DIGEST_LENGTH);

            http_builder builder;

            string response =
                builder.status(101)
                    .header("Upgrade: websocket")
                    .header("Connection: Upgrade")
                    .header("Sec-WebSocket-Accept: " + base64_key);

            int bytes = send(newfd, response.data(), response.size(), 0);

            if (bytes == -1) {
                perror("send error");
                close(newfd);
            }

            unsigned char buf[this->max_buf_size];
            int received = recv(newfd, buf, this->max_buf_size, 0);

            WebsocketFrame ws(buf, received);

            json json_response;
            json_response["id"] = utils::create_uuid();
            std::string json_str = json_response.dump();

            char response_buf[this->max_buf_size];

            uint8_t fin = 128;
            uint8_t opcode = 1;
            uint8_t fin_and_opcode = fin | opcode;

            uint8_t mask = 0;
            uint64_t payload_len = json_str.size();
            uint8_t pl;
            // we doing the same thing we did when we process the websocket
            // frame, just backwards this time
            if (payload_len <= 125) {
                pl = static_cast<uint8_t>(payload_len);
            }
            else if (payload_len <= 65535) { // 2^16
                pl = 126;
            }
            else { // <= 2^63
                pl = 127;
            }
            uint8_t mask_and_payloadlen = mask | pl;

            response_buf[0] = fin_and_opcode;
            response_buf[1] = mask_and_payloadlen;

            int bytes_written = 2;
            int payload_len_offset = 2;
            if (payload_len <= 125) {
            }
            else if (payload_len <= 65535) {
                uint16_t_converter temp;
                temp.i = htons(payload_len);
                response_buf[2] = temp.c[0];
                response_buf[3] = temp.c[1];
                bytes_written += 2;
                payload_len_offset = 4;
            }
            else { // MUST be <= 2^63 (the most sig. bit is 0)
                uint64_t_converter temp;
                temp.i = utils::_htonll(payload_len);
                response_buf[2] = temp.c[0];
                response_buf[3] = temp.c[1];
                response_buf[4] = temp.c[2];
                response_buf[5] = temp.c[3];
                response_buf[6] = temp.c[4];
                response_buf[7] = temp.c[5];
                response_buf[8] = temp.c[6];
                response_buf[9] = temp.c[7];
                bytes_written += 8;
                payload_len_offset = 10;
            }

            std::copy(json_str.begin(), json_str.end(),
                      response_buf + payload_len_offset);

            int bytes_sent =
                send(newfd, response_buf, bytes_written + json_str.size(), 0);
            if (bytes_sent == -1) {
                perror("byte sent error");
                continue;
            }
        }

        // auto route = this->router->find(req.path);
        // HTTP http(newfd, req);
        //
        // if (route) {
        //     auto route_handler = route->value;
        //
        //     if (route_handler) {
        //         if (route->isWildcard) {
        //             req.param = route->wildcardContent;
        //         }
        //         route_handler(req, http);
        //     }
        // }
        // else {
        //     std::cout << "route not found: " << req.path << endl;
        //     string response = http.not_found();
        //     int bytes = send(newfd, response.data(), response.size(), 0);
        //
        //     if (bytes == -1) {
        //         perror("send error");
        //     }
        // }

        close(newfd);
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
