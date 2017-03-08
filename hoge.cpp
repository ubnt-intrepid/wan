#!/usr/bin/env wan-script

#include <iostream>
#include <vector>

int main(int argc, char* argv[]) {
  auto args = std::vector<char*>(argv, argv + argc);
  std::cout << "Hoya" << std::endl;
  for (auto&& arg: args) {
    std::cout << arg << std::endl;
  }
}
