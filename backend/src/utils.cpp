#include "utils.h"
#include "assert.h"


vector<string> utils::split_str(string& str, string delimiters) {
    vector<string> tokens;

    string token = "";
    for (char &c : str) {
        if (delimiters.find(c) != std::string::npos) {
            if (!token.empty()) {
                tokens.push_back(token);
                token = "";
            }
        }
        else {
            token += c;
        }
    }

    if (!token.empty()) {
        tokens.push_back(token);
    }

    return tokens;
}

string utils::get_file_ext(string &filename) {
    auto tokens = utils::split_str(filename, ".");

    assert(tokens.size() == 2 && "Incorrect file name format");

    return tokens.back();
}
