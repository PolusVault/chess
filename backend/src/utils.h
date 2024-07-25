#pragma once

#include <vector>
#include <string>
#include "http.h"
using namespace std;

using Handler = void (*)(http_request &, HTTP &);

namespace utils {
vector<string> split_str(string &str, string delimiters);
string get_file_ext(string &filename);
string create_uuid(int len = 10);
string base64_encode(unsigned char const *bytes_to_encode, unsigned int in_len);
uint64_t _htonll(uint64_t src);
uint64_t _ntohll(uint64_t src);
} // namespace utils
