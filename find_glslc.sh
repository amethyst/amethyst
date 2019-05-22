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
