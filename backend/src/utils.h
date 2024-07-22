#include <vector>
#include <string>
#include "http.h"
using namespace std;

using Handler = void (*)(http_request &, HTTP &);

namespace utils {
vector<string> split_str(string &str, string delimiters);
string get_file_ext(string &filename);
} // namespace utils
