# 当前脚本用于初始化自定义的Rust工具链
if [ -z "$(which cargo)" ]; then
    echo "尚未安装Rust，请先安装Rust"
    exit 1
fi

# 是否强制覆盖已有的工具链配置文件
FORCE=0

while getopts "f" arg
do
    case $arg in
        f)
            FORCE=1
            ;;
        ?)
            echo "unkonw argument"
            exit 1
        ;;
    esac
done

DRAGONOS_UNKNOWN_ELF_PATH=$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-dragonos
mkdir -p ${DRAGONOS_UNKNOWN_ELF_PATH}/lib
echo $DRAGONOS_UNKNOWN_ELF_PATH

# 判断是否已经存在工具链配置文件
if [ -f "${DRAGONOS_UNKNOWN_ELF_PATH}/target.json" ]; then
    if [ $FORCE -eq 0 ]; then
        echo "已存在工具链配置文件，如需重新初始化，请使用-f参数"
        exit 0
    fi
fi

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