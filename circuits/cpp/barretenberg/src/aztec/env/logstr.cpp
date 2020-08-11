#include <iostream>

extern "C" {

void logstr(char const* str)
{
    std::cout << str << std::endl;
}

void logstr_err(char const* str)
{
    std::cerr << str << std::endl;
}
}