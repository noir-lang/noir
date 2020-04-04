#include <iostream>

extern "C" {

void logstr(char const* str)
{
    std::cout << str << std::endl;
}

}