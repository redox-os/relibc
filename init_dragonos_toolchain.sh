# 当前脚本用于初始化自定义的Rust工具链
if [ -z "$(which cargo)" ]; then
    echo "尚未安装Rust，请先安装Rust"
    exit 1
fi

DRAGONOS_UNKNOWN_ELF_PATH=$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-dragonos
mkdir -p ${DRAGONOS_UNKNOWN_ELF_PATH}/lib
echo $DRAGONOS_UNKNOWN_ELF_PATH
# 设置工具链配置文件
echo   \
"{\
    \"arch\": \"x86_64\",
    \"code-model\": \"kernel\",
    \"cpu\": \"x86-64\",
    \"os\": \"dragonos\",
    \"target-endian\": \"little\",
    \"target-pointer-width\": \"64\",
    \"target-c-int-width\": \"32\",
    \"data-layout\": \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\",
    \"disable-redzone\": true,
    \"features\": \"-3dnow,-3dnowa,-avx,-avx2\",
    \"linker\": \"rust-lld\",
    \"linker-flavor\": \"ld.lld\",
    \"llvm-target\": \"x86_64-unknown-none\",
    \"max-atomic-width\": 64,
    \"panic-strategy\": \"abort\",
    \"position-independent-executables\": true,
    \"relro-level\": \"full\",
    \"stack-probes\": {
      \"kind\": \"inline-or-call\",
      \"min-llvm-version-for-inline\": [
        16,
        0,
        0
      ]
    },
    \"static-position-independent-executables\": true,
    \"supported-sanitizers\": [
      \"kcfi\"
    ],
    \"target-pointer-width\": \"64\"
}" > ${DRAGONOS_UNKNOWN_ELF_PATH}/target.json || exit 1