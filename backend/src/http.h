#pragma once
#include <string>
#include <vector>
using namespace std;

// assume HTTP/1.1
struct http_request {
    string method;
    string path;
};

class HTTP {
    // TODO: move these into a "builder" class
    int _status;
    string _body;
    std::vector<string> _headers;

  public:
    HTTP();

    void process_request(char *buf, int newfd);

    // what does this "const" do?
    operator string() const;

    // builder methods
    // TODO: move these into a "builder" class
    HTTP *status(int code);
    HTTP *body(string body);
    HTTP *header(string header);
};
