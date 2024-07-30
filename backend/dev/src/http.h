#pragma once
#include <string>
#include <vector>
#include <map>

using std::string;

struct http_request {
    string method;
    string path;
    string param;

    std::map<string, string> headers;
    bool isWebsocketHandshake;
};

struct http_builder {
    int _status;
    string _body;
    string _version = "HTTP/1.1";
    std::vector<string> _headers;

    http_builder &status(int);
    http_builder &body(string);
    http_builder &header(string);

    // what does this "const" do?
    operator string() const;
};

class HTTP {
    int fd;
    http_request &req;

  public:
    static std::map<string, string> mime_types;
    HTTP(int fd, http_request &req);

    string not_found();
    string websocket_handshake();
    void sendFile(string fileName);
    void sendText(string text);
};
