#include <iostream>
#include <string>
#include <sstream>
#include <fstream>
#include <sys/socket.h>
#include "http.h"
#include "utils.h"
using namespace std;

HTTP::HTTP() {}

// TODO: parse headers too
http_request HTTP::process_request(char *buf)
{
    string http_msg(buf);
    http_request request;

    auto lines = utils::split_str(http_msg, "\r\n");
    auto tokens =
        utils::split_str(lines[0], " "); // lines[0] is the http startline

    request.method = tokens[0];
    request.path = tokens[1];

    return request;
}

void HTTP::handle_request(http_request& req, int fd) {
    // TODO: validate the paths
    auto path_tokens = utils::split_str(req.path, "/");
    string prefix = "../";
    string filename = "index.html";

    if (!path_tokens.empty()) {
        filename = path_tokens.back();
    }

    string response;
    http_builder builder;

    ifstream file(prefix + filename);
    std::stringstream content;

    if (file.is_open()) {
        content << file.rdbuf();
        string content_str = content.str();
        response = builder.status(200)
                       .body(content_str)
                       .header("Content-Type: text/html")
                       .header("Content-Length: " +
                               std::to_string(content_str.size()));
    }
    else {
        std::cout << "can't open file" << endl;
        string content_str = "404 Not Found";
        response = builder.status(404)
                       .body(content_str)
                       .header("Content-Type: text/plain")
                       .header("Content-Length: " +
                               std::to_string(content_str.size()));
    }

    int bytes = send(fd, response.data(), response.size(), 0);

    if (bytes == -1) {
        perror("send error");
    }
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
