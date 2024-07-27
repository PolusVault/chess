#include <string>
#include "websocket.h"
#include "network.h"
#include "utils.h"

ws::Data ws::parse_frame(unsigned char *buf)
{
    ws::Data data;
    data.is_close_frame = false;

    auto fin_and_opcode = buf[0];
    auto mask_and_length = buf[1];

    // clang-format off
    uint8_t opcode      = fin_and_opcode  & 0b00001111;
    uint8_t mask_code   = mask_and_length & 0b10000000; // mask is in the first bit
    uint8_t length      = mask_and_length & 0b01111111; // length is the rest of the bits after the first
    // clang-format on

    if (mask_code == 0) {
        // TODO: error, all frames coming from client should be masked
    }

    // check for Close frame
    if (opcode == 0x8) {
        data.is_close_frame = true;
        return data;
    }

    int mask_offset = 2;

    if (length == network::payload_size_code_16bit) {
        uint16_t_converter temp;
        std::copy(buf + 2, buf + 5, temp.c);
        length = ntohs(temp.i);
        mask_offset = 4;
    }
    else if (length == network::payload_size_code_64bit) {
        uint64_t_converter temp;
        std::copy(buf + 2, buf + 10, temp.c);
        length = utils::_ntohll(temp.i);
        mask_offset = 10;
    }

    unsigned char mask[4] = {buf[mask_offset], buf[mask_offset + 1],
                             buf[mask_offset + 2], buf[mask_offset + 3]};
    int payload_offset = mask_offset + 4;

    char msgbuf[length];
    for (int i = 0; i < length; i++) {
        // unmask the payload
        msgbuf[i] = buf[payload_offset + i] ^ mask[i % 4];
    }
    msgbuf[length] = '\0';

    data.payload = json::parse(msgbuf, msgbuf + length);

    return data;
}

char *ws::create_frame(string payload)
{
    char *response_buf = new char[4096];

    uint8_t fin = 128;
    uint8_t opcode = 1;
    uint8_t fin_and_opcode = fin | opcode;

    uint8_t mask = 0;
    uint64_t payload_len = payload.size();
    uint8_t pl;
    // we doing the same thing we did when we process the
    // websocket frame, just backwards this time
    if (payload_len <= network::SMALL_PAYLOAD_SIZE) {
        pl = static_cast<uint8_t>(payload_len);
    }
    else if (payload_len <= network::MEDIUM_PAYLOAD_SIZE) {
        pl = network::payload_size_code_16bit;
    }
    else {
        pl = network::payload_size_code_64bit;
    }
    uint8_t mask_and_payloadlen = mask | pl;

    response_buf[0] = fin_and_opcode;
    response_buf[1] = mask_and_payloadlen;

    int bytes_written = 2;
    int payload_len_offset = 2;

    if (payload_len <= network::SMALL_PAYLOAD_SIZE) {
        // do nothing
    }
    else if (payload_len <= network::MEDIUM_PAYLOAD_SIZE) {
        uint16_t_converter temp;
        temp.i = htons(payload_len);
        response_buf[2] = temp.c[0];
        response_buf[3] = temp.c[1];
        bytes_written += 2;
        payload_len_offset = 4;
    }
    else { // MUST be <= 2^63 (the most sig. bit is 0)
        uint64_t_converter temp;
        temp.i = utils::_htonll(payload_len);
        response_buf[2] = temp.c[0];
        response_buf[3] = temp.c[1];
        response_buf[4] = temp.c[2];
        response_buf[5] = temp.c[3];
        response_buf[6] = temp.c[4];
        response_buf[7] = temp.c[5];
        response_buf[8] = temp.c[6];
        response_buf[9] = temp.c[7];
        bytes_written += 8;
        payload_len_offset = 10;
    }

    std::copy(payload.begin(), payload.end(),
              response_buf + payload_len_offset);

    return response_buf;
}

char *ws::create_close_frame()
{
    char *response_buf = new char[2];

    uint8_t fin = 128;
    uint8_t opcode = 8;
    uint8_t fin_and_opcode = fin | opcode;

    uint8_t mask_and_payloadlen = 0;

    response_buf[0] = fin_and_opcode;
    response_buf[1] = mask_and_payloadlen;

    return response_buf;
}