cmake_minimum_required(VERSION 3.28)
project(chess_backend)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_STANDARD 14)

include_directories(.)

file(GLOB APP_SOURCES "src/*.cpp")
add_executable(chess_backend
    ${APP_SOURCES})
