#pragma once
#include <string>
#include <vector>
using namespace std;

struct http_request {
    string method;
    string path;
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
  public:
    HTTP();

    http_request process_request(char *);
    void handle_request(http_request &, int);
};
