cd "${0%/*}"
set -e

BIN=""

if [ -e "$(which glslc)" ]; then
    BIN="$(which glslc)"
elif [ -e "$(which glslc.exe)" ]; then
    # support WSL
    BIN="$(which glslc.exe)"
else
    >&2 echo "Cannot find glslc binary. Make sure you have VulkanSDK installed and added to PATH."
    exit 1
fi

echo "$BIN"

# for SHADER in $(find shaders -type f -name "*.vert" -o -name "*.frag"); do
#     echo $SHADER
#     "$BIN" -MD -c -g -O "$SHADER" -o "$SHADER.spv"
#     "$BIN" -S -g -O "$SHADER" -o "$SHADER.spvasm"
# done
