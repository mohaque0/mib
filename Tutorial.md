
## Hello, World!

To create a simple hello world program with dependencies you need 4 files:
```
/build.yml
/src/main.cpp
/hellolib/src/lib.hpp
/hellolib/src/lib.cpp
```

The build.yml should contain:
```
default:
  config:
    conan.user: helloworld
    conan.channel: stable
    conan.version: 0.1

module:
  - name: helloworld
    path: .
    deps:
      - hellolib
    config:
      conan.artifact_type: bin
      conan.requires:
        - hellolib/0.1@helloworld/stable
  - name: hellolib
    config:
      conan.artifact_name: hellolib
      conan.artifact_type: lib
```

This creates two modules: "helloworld" which produces a binary and "hellolib" which produces a library.
"helloworld" depends on "hellolib".

The rest of the files are a simple C++ hello world project.

"src/main.cpp":
```
#include "lib.hpp"
int main() {
	hello();
	return 0;
}
```

"hellolib/src/lib.hpp":
```
#ifndef __HELLO__
#define __HELLO__
void hello();
#endif // __HELLO__
```

"hellolib/src/lib.cpp":
```
#include <stdio.h>
void hello() {
	printf("Hello, World!\n");
}
```

In the project root run:
```
mib
```

You can also run:
```
mib build             # To build target 'build' which builds everything.
mib hellolib:build    # To build just "hellolib"
mib clean             # To clean all build directories of all modules.
mib helloworld:clean  # To clean just the build directory of the "helloworld" module.
```
