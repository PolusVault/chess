#pragma once

#include <vector>
#include <string>
#include "http.h"

#define WEBSOCKET_UUID_STRING "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"

using namespace std;
using Handler = void (*)(http_request &, HTTP &);

union uint16_t_converter {
    uint16_t i;
    uint8_t c[2];
};

union uint64_t_converter {
    uint64_t i;
    uint8_t c[8];
};

namespace utils {
vector<string> split_str(string &str, string delimiters);
string get_file_ext(string &filename);
string create_uuid(int len = 10);
string base64_encode(unsigned char const *bytes_to_encode, unsigned int in_len);
uint64_t _htonll(uint64_t src);
uint64_t _ntohll(uint64_t src);
} // namespace utils
