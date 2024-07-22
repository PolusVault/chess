#include <iostream>
#include <string>
#include <sstream>
#include <fstream>
#include <sys/socket.h>
#include "http.h"
#include "utils.h"
using namespace std;

std::map<string, string> HTTP::mime_types = {
    {"txt", "text/plain"},    {"html", "text/html"},
    {"svg", "image/svg+xml"}, {"wasm", "application/wasm"},
    {"css", "text/css"},      {"js", "text/javascript"}};

HTTP::HTTP(int sockfd, http_request &req) : req(req)
{
    this->fd = sockfd;
}

void HTTP::sendFile(string fileName)
{
    // TODO: validate the paths
    string prefix = "../";

    string response;
    http_builder builder;

    // ifstream file(prefix + fileName);
    ifstream file(fileName);
    std::stringstream content;
    string ext = utils::get_file_ext(fileName);

    if (file.is_open()) {
        content << file.rdbuf();
        string content_str = content.str();
        response = builder.status(200)
                       .body(content_str)
                       .header("Content-Type: " + HTTP::mime_types[ext])
                       .header("Content-Length: " +
                               std::to_string(content_str.size()));
    }
    else {
        std::cout << "can't open file" << endl;
        response = this->not_found();
    }

    int bytes = send(this->fd, response.data(), response.size(), 0);

    if (bytes == -1) {
        perror("send error");
    }
}

void HTTP::sendText(string text)
{
    http_builder builder;
    string response =
        builder.status(200)
            .body(text)
            .header("Content-Type: text/plain")
            .header("Content-Length: " + std::to_string(text.size()));

    int bytes = send(this->fd, response.data(), response.size(), 0);

    if (bytes == -1) {
        perror("send error");
    }
}

string HTTP::not_found()
{
    http_builder builder;

    string content_str = "404 Not Found";
    string response =
        builder.status(404)
            .body(content_str)
            .header("Content-Type: text/plain")
            .header("Content-Length: " + std::to_string(content_str.size()));
    return response;
}

http_builder::operator string() const
{
    string res = "";
    string status_line = std::to_string(this->_status);

    if (this->_status == 200) {
        status_line += " OK";
    }
    else {
        status_line += " Not Found";
    }

    res += this->_version + " " + status_line + "\r\n";

    // why do need const?
    for (const string &h : this->_headers) {
        res += h + "\r\n";
    }

    res += "\n";
    res += this->_body + "\r\n";
    res += "\r\n";

    return res;
}

http_builder &http_builder::status(int code)
{
    this->_status = code;
    return *this;
}

http_builder &http_builder::body(string content)
{
    this->_body = content;
    return *this;
}

http_builder &http_builder::header(string h)
{
    this->_headers.push_back(h);
    return *this;
}
