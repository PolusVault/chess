#pragma once
#include <string>

// using Handler = void (*)(http_request &, HTTP &);

namespace network {
static const std::string WEBSOCKET_UUID_STRING =
    "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
static const uint8_t SMALL_PAYLOAD_SIZE = 125;
static const uint16_t MEDIUM_PAYLOAD_SIZE = 0xFFFF;              // 2^16, 65535
static const uint64_t LARGE_PAYLOAD_SIZE = 0x7FFFFFFFFFFFFFFFLL; // 2^63
static const uint8_t payload_size_code_16bit = 0x7E;             // 126
static const uint8_t payload_size_code_64bit = 0x7F;             // 127
} // namespace network
