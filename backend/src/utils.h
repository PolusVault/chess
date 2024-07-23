#include <vector>
#include <string>
#include "http.h"
using namespace std;

using Handler = void (*)(http_request &, HTTP &);

namespace utils {
vector<string> split_str(string &str, string delimiters);
string get_file_ext(string &filename);
std::string base64_encode(unsigned char const* bytes_to_encode, unsigned int in_len);
} // namespace utils
