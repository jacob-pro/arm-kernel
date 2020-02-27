compiledb -n -o - make\
 | sed 's&"directory": "/c/&"directory": "c:/&'\
 | sed 's&/bin/arm-eabi-gcc&/bin/arm-eabi-gcc.exe&'\
 > compile_commands.json