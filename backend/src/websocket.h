#pragma once

#include <iostream>
#include <cstdint>
#include <string>
#include <nlohmann/json.hpp>
#include "utils.h"

using json = nlohmann::json;

struct WebsocketFrame {
    json payload;

    // parse the frame buffer
    WebsocketFrame(unsigned char *buf)
    {
        auto fin_and_opcode = buf[0];
        auto mask_and_length = buf[1];

        // clang-format off
        uint8_t mask_code   = mask_and_length & 0b10000000; // mask is in the first bit
        uint8_t length      = mask_and_length & 0b01111111; // length is the rest of the bits after the first
        // clang-format on

        if (mask_code == 0) {
            // TODO: error, all frames coming from client should be masked
        }

        int mask_offset = 2;

        if (length == 126) {
            uint16_t_converter temp;
            std::copy(buf + 2, buf + 5, temp.c);
            length = ntohs(temp.i);
            mask_offset = 4;
        }
        else if (length == 127) {
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

        this->payload = json::parse(msgbuf, msgbuf + length);

        std::cout << this->payload["id"] << endl;

        // TODO: create a message object from the json to use
        // see notes.md for formats
    };

    static char *createFrame(std::string payload)
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
        if (payload_len <= 125) {
            pl = static_cast<uint8_t>(payload_len);
        }
        else if (payload_len <= 65535) { // 2^16
            pl = 126;
        }
        else { // <= 2^63
            pl = 127;
        }
        uint8_t mask_and_payloadlen = mask | pl;

        response_buf[0] = fin_and_opcode;
        response_buf[1] = mask_and_payloadlen;

        int bytes_written = 2;
        int payload_len_offset = 2;
        if (payload_len <= 125) {
        }
        else if (payload_len <= 65535) {
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
};
