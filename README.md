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
- In an MSYS2 terminal `pacman --sync make`
- Make is installed at `C:\msys64\usr\bin\make.exe`

### CLion 

#### Terminal
- Use shell path `"C:\msys64\usr\bin\bash.exe" --login -i`
- Set env variable `CHERE_INVOKING=1` to launch console in correct directory