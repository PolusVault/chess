cmake_minimum_required(VERSION 3.28)
project(chess_backend)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_STANDARD 20)

include(FetchContent)
include_directories(.)

FetchContent_Declare(json URL https://github.com/nlohmann/json/releases/download/v3.11.3/json.tar.xz)
FetchContent_MakeAvailable(json)

set(OPENSSL_ROOT_DIR "/opt/homebrew/Cellar/openssl@3")
find_package(OpenSSL REQUIRED)

# find_package(spdlog REQUIRED PATHS "./lib")
find_package(spdlog REQUIRED PATHS "./lib/spdlog/build")

file(GLOB_RECURSE APP_SOURCES "src/*.cpp")
file(GLOB_RECURSE TRIE_SOURCES "src/trie/*.cpp")
add_executable(chess_backend
    ${APP_SOURCES} ${TRIE_SOURCES})
target_link_libraries(${PROJECT_NAME} PRIVATE OpenSSL::SSL OpenSSL::Crypto nlohmann_json::nlohmann_json
    spdlog::spdlog $<$<BOOL:${MINGW}>:ws2_32>)
