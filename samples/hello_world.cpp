#include <iostream>
#include <vector>
#include <boost/optional.hpp>
using boost::optional;

int main() {
  std::cout << boost::optional<int>{10} << std::endl;
  std::cout << boost::none << std::endl;
}
