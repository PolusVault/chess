#include <iostream>
#include <string>
#include <sstream>
#include <fstream>
#include "http.h"
#include "utils.h"
using namespace std;

HTTP::HTTP() {}

void HTTP::process_request(char *buf, int newfd)
{
    string http_msg(buf);
    http_request request;

    auto lines = utils::split_str(http_msg, "\r\n");
    auto tokens =
        utils::split_str(lines[0], " "); // lines[0] is the http startline

    request.method = tokens[0];
    request.path = tokens[1];

    // TODO: do this in handle_request
    ifstream file("../index.html");
    std::stringstream content;
    if (file.is_open()) {
        content << file.rdbuf();
    }
    else {
        std::cout << "can't open file" << endl;
    }

    string response =
        static_cast<string>(*(this->status(200)->body("test")->header("test")));
    
    std::cout << response << endl;
}

HTTP::operator string() const {
    return "test";
}

HTTP *HTTP::status(int code)
{
    this->_status = code;
    return this;
}

HTTP *HTTP::body(string content)
{
    this->_body = content;
    return this;
}

HTTP *HTTP::header(string h)
{
    this->_headers.push_back(h);
    return this;
}
