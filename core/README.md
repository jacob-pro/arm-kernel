## Running on Windows

### Linaro

Download Linaro for Windows (mingw32)\
https://releases.linaro.org/components/toolchain/binaries/latest-7/arm-eabi/

Linaro is missing 2 DLL dependencies required for `arm-eabi-gdb.exe`\
These are `libgcc_s_sjlj-1.dll` and `libstdc++-6.dll`

These are bundled with SJLJ (SetJump LongJump) builds of GCC\
https://sourceforge.net/projects/mingw-w64/files/Toolchains%20targetting%20Win32/\
Download the `i686-posix-sjlj` release\
Extract and put the two DLLs in `LINARO_PATH\bin`

### Make

- Download MSYS2 https://www.msys2.org/
- In an MSYS2 terminal `pacman -S make`

### CLion

#### Terminal
- Use shell path `"C:\msys64\usr\bin\bash.exe" --login -i`
- Set env variable `CHERE_INVOKING=1` to launch console in correct directory
- Set env variable `MSYS2_PATH_TYPE=inherit` to inherit Windows PATH variable

#### Compiler

Makefiles are supported via Compilation Database\
https://www.jetbrains.com/help/clion/managing-makefile-projects.html#

Inside MSYS:
- `pacman -S python3`
- `pacman -S python3-pip`
- `pip install compiledb`

Run `compiledb.sh` to generate `compile_commands.json`

Load CompilationDB project in Clion\
https://www.jetbrains.com/help/clion/compilation-database.html

#### Remote Debug

- Run/Debug Configurations -> GDB Remote Debug
- Use arm-eabi-gdb.exe
- Set target remote `127.0.0.1:1234`
- Set Symbol file `image.elf`
