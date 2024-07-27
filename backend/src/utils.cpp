#include <random>
#include "utils.h"
#include "assert.h"

std::vector<string> utils::split_str(string &str, string delimiters)
{
    std::vector<string> tokens;

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

string utils::get_file_ext(string &filename)
{
    auto tokens = utils::split_str(filename, ".");

    return tokens.back();
}

string utils::create_uuid(int len)
{
    static std::random_device dev;
    static std::mt19937 rng(dev());

    const char *v = "0123456789abcdef";
    std::uniform_int_distribution<int> dist(0, strlen(v) - 1);

    string res;
    for (int i = 0; i < len; i++) {
        res += v[dist(rng)];
    }
    return res;
}

static const std::string base64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                                        "abcdefghijklmnopqrstuvwxyz"
                                        "0123456789+/";

std::string utils::base64_encode(unsigned char const *bytes_to_encode,
                                 unsigned int in_len)
{
    std::string ret;
    int i = 0;
    int j = 0;
    unsigned char char_array_3[3];
    unsigned char char_array_4[4];

    while (in_len--) {
        char_array_3[i++] = *(bytes_to_encode++);
        if (i == 3) {
            char_array_4[0] = (char_array_3[0] & 0xfc) >> 2;
            char_array_4[1] = ((char_array_3[0] & 0x03) << 4) +
                              ((char_array_3[1] & 0xf0) >> 4);
            char_array_4[2] = ((char_array_3[1] & 0x0f) << 2) +
                              ((char_array_3[2] & 0xc0) >> 6);
            char_array_4[3] = char_array_3[2] & 0x3f;

            for (i = 0; (i < 4); i++)
                ret += base64_chars[char_array_4[i]];
            i = 0;
        }
    }

    if (i) {
        for (j = i; j < 3; j++)
            char_array_3[j] = '\0';

        char_array_4[0] = (char_array_3[0] & 0xfc) >> 2;
        char_array_4[1] =
            ((char_array_3[0] & 0x03) << 4) + ((char_array_3[1] & 0xf0) >> 4);
        char_array_4[2] =
            ((char_array_3[1] & 0x0f) << 2) + ((char_array_3[2] & 0xc0) >> 6);
        char_array_4[3] = char_array_3[2] & 0x3f;

        for (j = 0; (j < i + 1); j++)
            ret += base64_chars[char_array_4[j]];

        while ((i++ < 3))
            ret += '=';
    }

    return ret;
}

#define TYP_INIT 0
#define TYP_SMLE 1
#define TYP_BIGE 2

/// https://github.com/zaphoyd/websocketpp/blob/1b11fd301531e6df35a6107c1e8665b1e77a2d8e/websocketpp/common/network.hpp
/// Convert 64 bit value to network byte order
/**
 * @param src The integer in host byte order
 * @return src converted to network byte order
 */
uint64_t utils::_htonll(uint64_t src)
{
    static int typ = TYP_INIT;
    unsigned char c;
    union {
        uint64_t ull;
        unsigned char c[8];
    } x;
    if (typ == TYP_INIT) {
        x.ull = 0x01;
        typ = (x.c[7] == 0x01ULL) ? TYP_BIGE : TYP_SMLE;
    }
    if (typ == TYP_BIGE)
        return src;
    x.ull = src;
    c = x.c[0];
    x.c[0] = x.c[7];
    x.c[7] = c;
    c = x.c[1];
    x.c[1] = x.c[6];
    x.c[6] = c;
    c = x.c[2];
    x.c[2] = x.c[5];
    x.c[5] = c;
    c = x.c[3];
    x.c[3] = x.c[4];
    x.c[4] = c;
    return x.ull;
}

/// Convert 64 bit value to host byte order
/**
 * @param src The integer in network byte order
 * @return src converted to host byte order
 */
uint64_t utils::_ntohll(uint64_t src)
{
    return _htonll(src);
}
