#!/bin/bash

# Generates a CompileDB for use in an IDE

# We need to replace some paths to work in Windows
if [ "$(expr substr "$(uname -s)" 1 7)" == "MSYS_NT" ]; then
    echo "MSYS"
    compiledb -n -o - make\
   | sed 's&"directory": "/c/&"directory": "c:/&'\
   | sed 's&/bin/arm-eabi-gcc&/bin/arm-eabi-gcc.exe&'\
   > compile_commands.json
else
    echo "Non MSYS"
    compiledb -n -o - make > compile_commands.json
fi
