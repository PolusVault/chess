#pragma once

#include <string>
#include <nlohmann/json.hpp>

using std::string;
using json = nlohmann::json;

namespace ws {

struct Data {
    bool is_close_frame;
    json payload;
};

Data parse_frame(unsigned char *buf);
char *create_frame(string payload);
char *create_close_frame();
} // namespace ws
