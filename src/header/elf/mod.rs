use crate::platform::types::*;

pub type Elf32_Half = uint16_t;
pub type Elf64_Half = uint16_t;

pub type Elf32_Word = uint32_t;
pub type Elf32_Sword = int32_t;
pub type Elf64_Word = uint64_t;
pub type Elf64_Sword = int64_t;

pub type Elf32_Xword = uint64_t;
pub type Elf32_Sxword = int64_t;
pub type Elf64_Xword = uint64_t;
pub type Elf64_Sxword = int64_t;

pub type Elf32_Addr = uint32_t;
pub type Elf64_Addr = uint64_t;

pub type Elf32_Off = uint32_t;
pub type Elf64_Off = uint64_t;

pub type Elf32_Section = uint16_t;
pub type Elf64_Section = uint16_t;

pub type Elf32_Versym = Elf32_Half;
pub type Elf64_Versym = Elf64_Half;

pub const EI_NIDENT: usize = 16;

#[repr(C)]
pub struct Elf32_Ehdr {
    pub e_ident: [c_uchar; EI_NIDENT],
    pub e_type: Elf32_Half,
    pub e_machine: Elf32_Half,
    pub e_version: Elf32_Word,
    pub e_entry: Elf32_Addr,
    pub e_phoff: Elf32_Off,
    pub e_shoff: Elf32_Off,
    pub e_flags: Elf32_Word,
    pub e_ehsize: Elf32_Half,
    pub e_phentsize: Elf32_Half,
    pub e_phnum: Elf32_Half,
    pub e_shentsize: Elf32_Half,
    pub e_shnum: Elf32_Half,
    pub e_shstrndx: Elf32_Half,
}

#[repr(C)]
pub struct Elf64_Ehdr {
    pub e_ident: [c_uchar; EI_NIDENT],
    pub e_type: Elf64_Half,
    pub e_machine: Elf64_Half,
    pub e_version: Elf64_Word,
    pub e_entry: Elf64_Addr,
    pub e_phoff: Elf64_Off,
    pub e_shoff: Elf64_Off,
    pub e_flags: Elf64_Word,
    pub e_ehsize: Elf64_Half,
    pub e_phentsize: Elf64_Half,
    pub e_phnum: Elf64_Half,
    pub e_shentsize: Elf64_Half,
    pub e_shnum: Elf64_Half,
    pub e_shstrndx: Elf64_Half,
}

pub const EI_MAG0: usize = 0;
pub const ELFMAG0: usize = 0x7f;
pub const EI_MAG1: usize = 1;
pub const ELFMAG1: c_char = 'E' as c_char;
pub const EI_MAG2: usize = 2;
pub const ELFMAG2: c_char = 'L' as c_char;
pub const EI_MAG3: usize = 3;
pub const ELFMAG3: c_char = 'F' as c_char;
pub const ELFMAG: &'static str = "\x7fELF";
pub const SELFMAG: usize = 4;
pub const EI_CLASS: usize = 4;
pub const ELFCLASSNONE: usize = 0;
pub const ELFCLASS32: usize = 1;
pub const ELFCLASS64: usize = 2;
pub const ELFCLASSNUM: usize = 3;
pub const EI_DATA: usize = 5;
pub const ELFDATANONE: usize = 0;
pub const ELFDATA2LSB: usize = 1;
pub const ELFDATA2MSB: usize = 2;
pub const ELFDATANUM: usize = 3;
pub const EI_VERSION: usize = 6;
pub const EI_OSABI: usize = 7;
pub const ELFOSABI_NONE: usize = 0;
pub const ELFOSABI_SYSV: usize = 0;
pub const ELFOSABI_HPUX: usize = 1;
pub const ELFOSABI_NETBSD: usize = 2;
pub const ELFOSABI_LINUX: usize = 3;
pub const ELFOSABI_GNU: usize = 3;
pub const ELFOSABI_SOLARIS: usize = 6;
pub const ELFOSABI_AIX: usize = 7;
pub const ELFOSABI_IRIX: usize = 8;
pub const ELFOSABI_FREEBSD: usize = 9;
pub const ELFOSABI_TRU64: usize = 10;
pub const ELFOSABI_MODESTO: usize = 11;
pub const ELFOSABI_OPENBSD: usize = 12;
pub const ELFOSABI_ARM: usize = 97;
pub const ELFOSABI_STANDALONE: usize = 255;
pub const EI_ABIVERSION: usize = 8;
pub const EI_PAD: usize = 9;
pub const ET_NONE: usize = 0;
pub const ET_REL: usize = 1;
pub const ET_EXEC: usize = 2;
pub const ET_DYN: usize = 3;
pub const ET_CORE: usize = 4;
pub const ET_NUM: usize = 5;
pub const ET_LOOS: usize = 0xfe00;
pub const ET_HIOS: usize = 0xfeff;
pub const ET_LOPROC: usize = 0xff00;
pub const ET_HIPROC: usize = 0xffff;
pub const EM_NONE: usize = 0;
pub const EM_M32: usize = 1;
pub const EM_SPARC: usize = 2;
pub const EM_386: usize = 3;
pub const EM_68K: usize = 4;
pub const EM_88K: usize = 5;
pub const EM_860: usize = 7;
pub const EM_MIPS: usize = 8;
pub const EM_S370: usize = 9;
pub const EM_MIPS_RS3_LE: usize = 10;
pub const EM_PARISC: usize = 15;
pub const EM_VPP500: usize = 17;
pub const EM_SPARC32PLUS: usize = 18;
pub const EM_960: usize = 19;
pub const EM_PPC: usize = 20;
pub const EM_PPC64: usize = 21;
pub const EM_S390: usize = 22;
pub const EM_V800: usize = 36;
pub const EM_FR20: usize = 37;
pub const EM_RH32: usize = 38;
pub const EM_RCE: usize = 39;
pub const EM_ARM: usize = 40;
pub const EM_FAKE_ALPHA: usize = 41;
pub const EM_SH: usize = 42;
pub const EM_SPARCV9: usize = 43;
pub const EM_TRICORE: usize = 44;
pub const EM_ARC: usize = 45;
pub const EM_H8_300: usize = 46;
pub const EM_H8_300H: usize = 47;
pub const EM_H8S: usize = 48;
pub const EM_H8_500: usize = 49;
pub const EM_IA_64: usize = 50;
pub const EM_MIPS_X: usize = 51;
pub const EM_COLDFIRE: usize = 52;
pub const EM_68HC12: usize = 53;
pub const EM_MMA: usize = 54;
pub const EM_PCP: usize = 55;
pub const EM_NCPU: usize = 56;
pub const EM_NDR1: usize = 57;
pub const EM_STARCORE: usize = 58;
pub const EM_ME16: usize = 59;
pub const EM_ST100: usize = 60;
pub const EM_TINYJ: usize = 61;
pub const EM_X86_64: usize = 62;
pub const EM_PDSP: usize = 63;
pub const EM_FX66: usize = 66;
pub const EM_ST9PLUS: usize = 67;
pub const EM_ST7: usize = 68;
pub const EM_68HC16: usize = 69;
pub const EM_68HC11: usize = 70;
pub const EM_68HC08: usize = 71;
pub const EM_68HC05: usize = 72;
pub const EM_SVX: usize = 73;
pub const EM_ST19: usize = 74;
pub const EM_VAX: usize = 75;
pub const EM_CRIS: usize = 76;
pub const EM_JAVELIN: usize = 77;
pub const EM_FIREPATH: usize = 78;
pub const EM_ZSP: usize = 79;
pub const EM_MMIX: usize = 80;
pub const EM_HUANY: usize = 81;
pub const EM_PRISM: usize = 82;
pub const EM_AVR: usize = 83;
pub const EM_FR30: usize = 84;
pub const EM_D10V: usize = 85;
pub const EM_D30V: usize = 86;
pub const EM_V850: usize = 87;
pub const EM_M32R: usize = 88;
pub const EM_MN10300: usize = 89;
pub const EM_MN10200: usize = 90;
pub const EM_PJ: usize = 91;
pub const EM_OR1K: usize = 92;
pub const EM_ARC_A5: usize = 93;
pub const EM_XTENSA: usize = 94;
pub const EM_AARCH64: usize = 183;
pub const EM_TILEPRO: usize = 188;
pub const EM_MICROBLAZE: usize = 189;
pub const EM_TILEGX: usize = 191;
pub const EM_NUM: usize = 192;
pub const EM_ALPHA: usize = 0x9026;
pub const EV_NONE: usize = 0;
pub const EV_CURRENT: usize = 1;
pub const EV_NUM: usize = 2;

#[repr(C)]
pub struct Elf32_Shdr {
    pub sh_name: Elf32_Word,
    pub sh_type: Elf32_Word,
    pub sh_flags: Elf32_Word,
    pub sh_addr: Elf32_Addr,
    pub sh_offset: Elf32_Off,
    pub sh_size: Elf32_Word,
    pub sh_link: Elf32_Word,
    pub sh_info: Elf32_Word,
    pub sh_addralign: Elf32_Word,
    pub sh_entsize: Elf32_Word,
}

#[repr(C)]
pub struct Elf64_Shdr {
    pub sh_name: Elf64_Word,
    pub sh_type: Elf64_Word,
    pub sh_flags: Elf64_Xword,
    pub sh_addr: Elf64_Addr,
    pub sh_offset: Elf64_Off,
    pub sh_size: Elf64_Xword,
    pub sh_link: Elf64_Word,
    pub sh_info: Elf64_Word,
    pub sh_addralign: Elf64_Xword,
    pub sh_entsize: Elf64_Xword,
}

pub const SHN_UNDEF: usize = 0;
pub const SHN_LORESERVE: usize = 0xff00;
pub const SHN_LOPROC: usize = 0xff00;
pub const SHN_BEFORE: usize = 0xff00;
pub const SHN_AFTER: usize = 0xff01;
pub const SHN_HIPROC: usize = 0xff1f;
pub const SHN_LOOS: usize = 0xff20;
pub const SHN_HIOS: usize = 0xff3f;
pub const SHN_ABS: usize = 0xfff1;
pub const SHN_COMMON: usize = 0xfff2;
pub const SHN_XINDEX: usize = 0xffff;
pub const SHN_HIRESERVE: usize = 0xffff;
pub const SHT_NULL: usize = 0;
pub const SHT_PROGBITS: usize = 1;
pub const SHT_SYMTAB: usize = 2;
pub const SHT_STRTAB: usize = 3;
pub const SHT_RELA: usize = 4;
pub const SHT_HASH: usize = 5;
pub const SHT_DYNAMIC: usize = 6;
pub const SHT_NOTE: usize = 7;
pub const SHT_NOBITS: usize = 8;
pub const SHT_REL: usize = 9;
pub const SHT_SHLIB: usize = 10;
pub const SHT_DYNSYM: usize = 11;
pub const SHT_INIT_ARRAY: usize = 14;
pub const SHT_FINI_ARRAY: usize = 15;
pub const SHT_PREINIT_ARRAY: usize = 16;
pub const SHT_GROUP: usize = 17;
pub const SHT_SYMTAB_SHNDX: usize = 18;
pub const SHT_NUM: usize = 19;
pub const SHT_LOOS: usize = 0x60000000;
pub const SHT_GNU_ATTRIBUTES: usize = 0x6ffffff5;
pub const SHT_GNU_HASH: usize = 0x6ffffff6;
pub const SHT_GNU_LIBLIST: usize = 0x6ffffff7;
pub const SHT_CHECKSUM: usize = 0x6ffffff8;
pub const SHT_LOSUNW: usize = 0x6ffffffa;
pub const SHT_SUNW_move: usize = 0x6ffffffa;
pub const SHT_SUNW_COMDAT: usize = 0x6ffffffb;
pub const SHT_SUNW_syminfo: usize = 0x6ffffffc;
pub const SHT_GNU_verdef: usize = 0x6ffffffd;
pub const SHT_GNU_verneed: usize = 0x6ffffffe;
pub const SHT_GNU_versym: usize = 0x6fffffff;
pub const SHT_HISUNW: usize = 0x6fffffff;
pub const SHT_HIOS: usize = 0x6fffffff;
pub const SHT_LOPROC: usize = 0x70000000;
pub const SHT_HIPROC: usize = 0x7fffffff;
pub const SHT_LOUSER: usize = 0x80000000;
pub const SHT_HIUSER: usize = 0x8fffffff;

pub const SHF_WRITE: usize = 1 << 0;
pub const SHF_ALLOC: usize = 1 << 1;
pub const SHF_EXECINSTR: usize = 1 << 2;
pub const SHF_MERGE: usize = 1 << 4;
pub const SHF_STRINGS: usize = 1 << 5;
pub const SHF_INFO_LINK: usize = 1 << 6;
pub const SHF_LINK_ORDER: usize = 1 << 7;
pub const SHF_OS_NONCONFORMING: usize = 1 << 8;
pub const SHF_GROUP: usize = 1 << 9;
pub const SHF_TLS: usize = 1 << 10;
pub const SHF_MASKOS: usize = 0x0ff00000;
pub const SHF_MASKPROC: usize = 0xf0000000;
pub const SHF_ORDERED: usize = 1 << 30;
pub const SHF_EXCLUDE: usize = 1 << 31;
pub const GRP_COMDAT: usize = 0x1;

#[repr(C)]
pub struct Elf32_Sym {
    pub st_name: Elf32_Word,
    pub st_value: Elf32_Addr,
    pub st_size: Elf32_Word,
    pub st_info: c_uchar,
    pub st_other: c_uchar,
    pub st_shndx: Elf32_Section,
}

#[repr(C)]
pub struct Elf64_Sym {
    pub st_name: Elf64_Word,
    pub st_info: c_uchar,
    pub st_other: c_uchar,
    pub st_shndx: Elf64_Section,
    pub st_value: Elf64_Addr,
    pub st_size: Elf64_Xword,
}

#[repr(C)]
pub struct Elf32_Syminfo {
    pub si_boundto: Elf32_Half,
    pub si_flags: Elf32_Half,
}

#[repr(C)]
pub struct Elf64_Syminfo {
    pub si_boundto: Elf64_Half,
    pub si_flags: Elf64_Half,
}

pub const SYMINFO_BT_SELF: usize = 0xffff;
pub const SYMINFO_BT_PARENT: usize = 0xfffe;
pub const SYMINFO_BT_LOWRESERVE: usize = 0xff00;
pub const SYMINFO_FLG_DIRECT: usize = 0x0001;
pub const SYMINFO_FLG_PASSTHRU: usize = 0x0002;
pub const SYMINFO_FLG_COPY: usize = 0x0004;
pub const SYMINFO_FLG_LAZYLOAD: usize = 0x0008;
pub const SYMINFO_NONE: usize = 0;
pub const SYMINFO_CURRENT: usize = 1;
pub const SYMINFO_NUM: usize = 2;

pub const STB_LOCAL: usize = 0;
pub const STB_GLOBAL: usize = 1;
pub const STB_WEAK: usize = 2;
pub const STB_NUM: usize = 3;
pub const STB_LOOS: usize = 10;
pub const STB_GNU_UNIQUE: usize = 10;
pub const STB_HIOS: usize = 12;
pub const STB_LOPROC: usize = 13;
pub const STB_HIPROC: usize = 15;
pub const STT_NOTYPE: usize = 0;
pub const STT_OBJECT: usize = 1;
pub const STT_FUNC: usize = 2;
pub const STT_SECTION: usize = 3;
pub const STT_FILE: usize = 4;
pub const STT_COMMON: usize = 5;
pub const STT_TLS: usize = 6;
pub const STT_NUM: usize = 7;
pub const STT_LOOS: usize = 10;
pub const STT_GNU_IFUNC: usize = 10;
pub const STT_HIOS: usize = 12;
pub const STT_LOPROC: usize = 13;
pub const STT_HIPROC: usize = 15;
pub const STN_UNDEF: usize = 0;

pub const STV_DEFAULT: usize = 0;
pub const STV_INTERNAL: usize = 1;
pub const STV_HIDDEN: usize = 2;
pub const STV_PROTECTED: usize = 3;

#[repr(C)]
pub struct Elf32_Rel {
    pub r_offset: Elf32_Addr,
    pub r_info: Elf32_Word,
}

#[repr(C)]
pub struct Elf64_Rel {
    pub r_offset: Elf64_Addr,
    pub r_info: Elf64_Xword,
}

#[repr(C)]
pub struct Elf32_Rela {
    pub r_offset: Elf32_Addr,
    pub r_info: Elf32_Word,
    pub r_addend: Elf32_Sword,
}

#[repr(C)]
pub struct Elf64_Rela {
    pub r_offset: Elf64_Addr,
    pub r_info: Elf64_Xword,
    pub r_addend: Elf64_Sxword,
}

#[repr(C)]
pub struct Elf32_Phdr {
    pub p_type: Elf32_Word,
    pub p_offset: Elf32_Off,
    pub p_vaddr: Elf32_Addr,
    pub p_paddr: Elf32_Addr,
    pub p_filesz: Elf32_Word,
    pub p_memsz: Elf32_Word,
    pub p_flags: Elf32_Word,
    pub p_align: Elf32_Word,
}

#[repr(C)]
pub struct Elf64_Phdr {
    pub p_type: Elf64_Word,
    pub p_flags: Elf64_Word,
    pub p_offset: Elf64_Off,
    pub p_vaddr: Elf64_Addr,
    pub p_paddr: Elf64_Addr,
    pub p_filesz: Elf64_Xword,
    pub p_memsz: Elf64_Xword,
    pub p_align: Elf64_Xword,
}

pub const PT_NULL: usize = 0;
pub const PT_LOAD: usize = 1;
pub const PT_DYNAMIC: usize = 2;
pub const PT_INTERP: usize = 3;
pub const PT_NOTE: usize = 4;
pub const PT_SHLIB: usize = 5;
pub const PT_PHDR: usize = 6;
pub const PT_TLS: usize = 7;
pub const PT_NUM: usize = 8;
pub const PT_LOOS: usize = 0x60000000;
pub const PT_GNU_EH_FRAME: usize = 0x6474e550;
pub const PT_GNU_STACK: usize = 0x6474e551;
pub const PT_GNU_RELRO: usize = 0x6474e552;
pub const PT_LOSUNW: usize = 0x6ffffffa;
pub const PT_SUNWBSS: usize = 0x6ffffffa;
pub const PT_SUNWSTACK: usize = 0x6ffffffb;
pub const PT_HISUNW: usize = 0x6fffffff;
pub const PT_HIOS: usize = 0x6fffffff;
pub const PT_LOPROC: usize = 0x70000000;
pub const PT_HIPROC: usize = 0x7fffffff;
pub const PN_XNUM: usize = 0xffff;
pub const PF_X: usize = 1 << 0;
pub const PF_W: usize = 1 << 1;
pub const PF_R: usize = 1 << 2;
pub const PF_MASKOS: usize = 0x0ff00000;
pub const PF_MASKPROC: usize = 0xf0000000;

pub const NT_PRSTATUS: usize = 1;
pub const NT_FPREGSET: usize = 2;
pub const NT_PRPSINFO: usize = 3;
pub const NT_PRXREG: usize = 4;
pub const NT_TASKSTRUCT: usize = 4;
pub const NT_PLATFORM: usize = 5;
pub const NT_AUXV: usize = 6;
pub const NT_GWINDOWS: usize = 7;
pub const NT_ASRS: usize = 8;
pub const NT_PSTATUS: usize = 10;
pub const NT_PSINFO: usize = 13;
pub const NT_PRCRED: usize = 14;
pub const NT_UTSNAME: usize = 15;
pub const NT_LWPSTATUS: usize = 16;
pub const NT_LWPSINFO: usize = 17;
pub const NT_PRFPXREG: usize = 20;
pub const NT_SIGINFO: usize = 0x53494749;
pub const NT_FILE: usize = 0x46494c45;
pub const NT_PRXFPREG: usize = 0x46e62b7f;
pub const NT_PPC_VMX: usize = 0x100;
pub const NT_PPC_SPE: usize = 0x101;
pub const NT_PPC_VSX: usize = 0x102;
pub const NT_386_TLS: usize = 0x200;
pub const NT_386_IOPERM: usize = 0x201;
pub const NT_X86_XSTATE: usize = 0x202;
pub const NT_S390_HIGH_GPRS: usize = 0x300;
pub const NT_S390_TIMER: usize = 0x301;
pub const NT_S390_TODCMP: usize = 0x302;
pub const NT_S390_TODPREG: usize = 0x303;
pub const NT_S390_CTRS: usize = 0x304;
pub const NT_S390_PREFIX: usize = 0x305;
pub const NT_S390_LAST_BREAK: usize = 0x306;
pub const NT_S390_SYSTEM_CALL: usize = 0x307;
pub const NT_S390_TDB: usize = 0x308;
pub const NT_ARM_VFP: usize = 0x400;
pub const NT_ARM_TLS: usize = 0x401;
pub const NT_ARM_HW_BREAK: usize = 0x402;
pub const NT_ARM_HW_WATCH: usize = 0x403;
pub const NT_METAG_CBUF: usize = 0x500;
pub const NT_METAG_RPIPE: usize = 0x501;
pub const NT_METAG_TLS: usize = 0x502;
pub const NT_VERSION: usize = 1;

#[repr(C)]
pub union Elf32_Dyn_Union {
    d_val: Elf32_Word,
    d_ptr: Elf32_Addr,
}

#[repr(C)]
pub struct Elf32_Dyn {
    pub d_tag: Elf32_Sword,
    pub d_un: Elf32_Dyn_Union,
}

#[repr(C)]
pub union Elf64_Dyn_Union {
    d_val: Elf64_Xword,
    d_ptr: Elf64_Addr,
}

#[repr(C)]
pub struct Elf64_Dyn {
    pub d_tag: Elf64_Sxword,
    pub d_un: Elf64_Dyn_Union,
}

pub const DT_NULL: usize = 0;
pub const DT_NEEDED: usize = 1;
pub const DT_PLTRELSZ: usize = 2;
pub const DT_PLTGOT: usize = 3;
pub const DT_HASH: usize = 4;
pub const DT_STRTAB: usize = 5;
pub const DT_SYMTAB: usize = 6;
pub const DT_RELA: usize = 7;
pub const DT_RELASZ: usize = 8;
pub const DT_RELAENT: usize = 9;
pub const DT_STRSZ: usize = 10;
pub const DT_SYMENT: usize = 11;
pub const DT_INIT: usize = 12;
pub const DT_FINI: usize = 13;
pub const DT_SONAME: usize = 14;
pub const DT_RPATH: usize = 15;
pub const DT_SYMBOLIC: usize = 16;
pub const DT_REL: usize = 17;
pub const DT_RELSZ: usize = 18;
pub const DT_RELENT: usize = 19;
pub const DT_PLTREL: usize = 20;
pub const DT_DEBUG: usize = 21;
pub const DT_TEXTREL: usize = 22;
pub const DT_JMPREL: usize = 23;
pub const DT_BIND_NOW: usize = 24;
pub const DT_INIT_ARRAY: usize = 25;
pub const DT_FINI_ARRAY: usize = 26;
pub const DT_INIT_ARRAYSZ: usize = 27;
pub const DT_FINI_ARRAYSZ: usize = 28;
pub const DT_RUNPATH: usize = 29;
pub const DT_FLAGS: usize = 30;
pub const DT_ENCODING: usize = 32;
pub const DT_PREINIT_ARRAY: usize = 32;
pub const DT_PREINIT_ARRAYSZ: usize = 33;
pub const DT_NUM: usize = 34;
pub const DT_LOOS: usize = 0x6000000d;
pub const DT_HIOS: usize = 0x6ffff000;
pub const DT_LOPROC: usize = 0x70000000;
pub const DT_HIPROC: usize = 0x7fffffff;
pub const DT_VALRNGLO: usize = 0x6ffffd00;
pub const DT_GNU_PRELINKED: usize = 0x6ffffdf5;
pub const DT_GNU_CONFLICTSZ: usize = 0x6ffffdf6;
pub const DT_GNU_LIBLISTSZ: usize = 0x6ffffdf7;
pub const DT_CHECKSUM: usize = 0x6ffffdf8;
pub const DT_PLTPADSZ: usize = 0x6ffffdf9;
pub const DT_MOVEENT: usize = 0x6ffffdfa;
pub const DT_MOVESZ: usize = 0x6ffffdfb;
pub const DT_FEATURE_1: usize = 0x6ffffdfc;
pub const DT_POSFLAG_1: usize = 0x6ffffdfd;
pub const DT_SYMINSZ: usize = 0x6ffffdfe;
pub const DT_SYMINENT: usize = 0x6ffffdff;
pub const DT_VALNUM: usize = 12;

pub const DT_ADDRRNGLO: usize = 0x6ffffe00;
pub const DT_GNU_HASH: usize = 0x6ffffef5;
pub const DT_TLSDESC_PLT: usize = 0x6ffffef6;
pub const DT_TLSDESC_GOT: usize = 0x6ffffef7;
pub const DT_GNU_CONFLICT: usize = 0x6ffffef8;
pub const DT_GNU_LIBLIST: usize = 0x6ffffef9;
pub const DT_CONFIG: usize = 0x6ffffefa;
pub const DT_DEPAUDIT: usize = 0x6ffffefb;
pub const DT_AUDIT: usize = 0x6ffffefc;
pub const DT_PLTPAD: usize = 0x6ffffefd;
pub const DT_MOVETAB: usize = 0x6ffffefe;
pub const DT_SYMINFO: usize = 0x6ffffeff;
pub const DT_ADDRNUM: usize = 11;
pub const DT_VERSYM: usize = 0x6ffffff0;
pub const DT_RELACOUNT: usize = 0x6ffffff9;
pub const DT_RELCOUNT: usize = 0x6ffffffa;
pub const DT_FLAGS_1: usize = 0x6ffffffb;
pub const DT_VERDEF: usize = 0x6ffffffc;
pub const DT_VERDEFNUM: usize = 0x6ffffffd;
pub const DT_VERNEED: usize = 0x6ffffffe;
pub const DT_VERSIONTAGNUM: usize = 16;
pub const DT_AUXILIARY: usize = 0x7ffffffd;
pub const DT_FILTER: usize = 0x7fffffff;
pub const DT_EXTRANUM: usize = 3;
pub const DF_ORIGIN: usize = 0x00000001;
pub const DF_SYMBOLIC: usize = 0x00000002;
pub const DF_TEXTREL: usize = 0x00000004;
pub const DF_BIND_NOW: usize = 0x00000008;
pub const DF_STATIC_TLS: usize = 0x00000010;
pub const DF_1_NOW: usize = 0x00000001;
pub const DF_1_GLOBAL: usize = 0x00000002;
pub const DF_1_GROUP: usize = 0x00000004;
pub const DF_1_NODELETE: usize = 0x00000008;
pub const DF_1_LOADFLTR: usize = 0x00000010;
pub const DF_1_INITFIRST: usize = 0x00000020;
pub const DF_1_NOOPEN: usize = 0x00000040;
pub const DF_1_ORIGIN: usize = 0x00000080;
pub const DF_1_DIRECT: usize = 0x00000100;
pub const DF_1_TRANS: usize = 0x00000200;
pub const DF_1_INTERPOSE: usize = 0x00000400;
pub const DF_1_NODEFLIB: usize = 0x00000800;
pub const DF_1_NODUMP: usize = 0x00001000;
pub const DF_1_CONFALT: usize = 0x00002000;
pub const DF_1_ENDFILTEE: usize = 0x00004000;
pub const DF_1_DISPRELDNE: usize = 0x00008000;
pub const DF_1_DISPRELPND: usize = 0x00010000;
pub const DF_1_NODIRECT: usize = 0x00020000;
pub const DF_1_IGNMULDEF: usize = 0x00040000;
pub const DF_1_NOKSYMS: usize = 0x00080000;
pub const DF_1_NOHDR: usize = 0x00100000;
pub const DF_1_EDITED: usize = 0x00200000;
pub const DF_1_NORELOC: usize = 0x00400000;
pub const DF_1_SYMINTPOSE: usize = 0x00800000;
pub const DF_1_GLOBAUDIT: usize = 0x01000000;
pub const DF_1_SINGLETON: usize = 0x02000000;
pub const DTF_1_PARINIT: usize = 0x00000001;
pub const DTF_1_CONFEXP: usize = 0x00000002;
pub const DF_P1_LAZYLOAD: usize = 0x00000001;
pub const DF_P1_GROUPPERM: usize = 0x00000002;

#[repr(C)]
pub struct Elf32_Verdef {
    pub vd_version: Elf32_Half,
    pub vd_flags: Elf32_Half,
    pub vd_ndx: Elf32_Half,
    pub vd_cnt: Elf32_Half,
    pub vd_hash: Elf32_Word,
    pub vd_aux: Elf32_Word,
    pub vd_next: Elf32_Word,
}

#[repr(C)]
pub struct Elf64_Verdef {
    pub vd_version: Elf64_Half,
    pub vd_flags: Elf64_Half,
    pub vd_ndx: Elf64_Half,
    pub vd_cnt: Elf64_Half,
    pub vd_hash: Elf64_Word,
    pub vd_aux: Elf64_Word,
    pub vd_next: Elf64_Word,
}

pub const VER_DEF_NONE: usize = 0;
pub const VER_DEF_CURRENT: usize = 1;
pub const VER_DEF_NUM: usize = 2;

pub const VER_FLG_BASE: usize = 0x1;
pub const VER_FLG_WEAK: usize = 0x2;
pub const VER_NDX_LOCAL: usize = 0;
pub const VER_NDX_GLOBAL: usize = 1;
pub const VER_NDX_LORESERVE: usize = 0xff00;
pub const VER_NDX_ELIMINATE: usize = 0xff01;

#[repr(C)]
pub struct Elf32_Verdaux {
    pub vda_name: Elf32_Word,
    pub vda_next: Elf32_Word,
}

#[repr(C)]
pub struct Elf64_Verdaux {
    pub vda_name: Elf64_Word,
    pub vda_next: Elf64_Word,
}

#[repr(C)]
pub struct Elf32_Verneed {
    pub vn_version: Elf32_Half,
    pub vn_cnt: Elf32_Half,
    pub vn_file: Elf32_Word,
    pub vn_aux: Elf32_Word,
    pub vn_next: Elf32_Word,
}

#[repr(C)]
pub struct Elf64_Verneed {
    pub vn_version: Elf64_Half,
    pub vn_cnt: Elf64_Half,
    pub vn_file: Elf64_Word,
    pub vn_aux: Elf64_Word,
    pub vn_next: Elf64_Word,
}

pub const VER_NEED_NONE: usize = 0;
pub const VER_NEED_CURRENT: usize = 1;
pub const VER_NEED_NUM: usize = 2;

#[repr(C)]
pub struct Elf64_Vernaux {
    pub vna_hash: Elf64_Word,
    pub vna_flags: Elf64_Half,
    pub vna_other: Elf64_Half,
    pub vna_name: Elf64_Word,
    pub vna_next: Elf64_Word,
}

#[repr(C)]
pub union A_UN {
    a_val: uint64_t,
}

#[repr(C)]
pub struct Elf64_auxv_t {
    pub a_type: uint64_t,
    pub a_un: A_UN,
}

pub const AT_NULL: usize = 0;
pub const AT_IGNORE: usize = 1;
pub const AT_EXECFD: usize = 2;
pub const AT_PHDR: usize = 3;
pub const AT_PHENT: usize = 4;
pub const AT_PHNUM: usize = 5;
pub const AT_PAGESZ: usize = 6;
pub const AT_BASE: usize = 7;
pub const AT_FLAGS: usize = 8;
pub const AT_ENTRY: usize = 9;
pub const AT_NOTELF: usize = 10;
pub const AT_UID: usize = 11;
pub const AT_EUID: usize = 12;
pub const AT_GID: usize = 13;
pub const AT_EGID: usize = 14;
pub const AT_CLKTCK: usize = 17;
pub const AT_PLATFORM: usize = 15;
pub const AT_HWCAP: usize = 16;
pub const AT_FPUCW: usize = 18;
pub const AT_DCACHEBSIZE: usize = 19;
pub const AT_ICACHEBSIZE: usize = 20;
pub const AT_UCACHEBSIZE: usize = 21;
pub const AT_IGNOREPPC: usize = 22;
pub const AT_SECURE: usize = 23;
pub const AT_BASE_PLATFORM: usize = 24;
pub const AT_RANDOM: usize = 25;
pub const AT_HWCAP2: usize = 26;
pub const AT_EXECFN: usize = 31;
pub const AT_SYSINFO: usize = 32;
pub const AT_SYSINFO_EHDR: usize = 33;
pub const AT_L1I_CACHESHAPE: usize = 34;
pub const AT_L1D_CACHESHAPE: usize = 35;
pub const AT_L2_CACHESHAPE: usize = 36;
pub const AT_L3_CACHESHAPE: usize = 37;

#[repr(C)]
pub struct Elf32_Nhdr {
    pub n_namesz: Elf32_Word,
    pub n_descsz: Elf32_Word,
    pub n_type: Elf32_Word,
}

#[repr(C)]
pub struct Elf64_Nhdr {
    pub n_namesz: Elf64_Word,
    pub n_descsz: Elf64_Word,
    pub n_type: Elf64_Word,
}

pub const ELF_NOTE_SOLARIS: &'static str = "SUNW Solaris";
pub const ELF_NOTE_GNU: &'static str = "GNU";

pub const ELF_NOTE_PAGESIZE_HINT: usize = 1;

pub const NT_GNU_ABI_TAG: usize = 1;
pub const ELF_NOTE_ABI: usize = NT_GNU_ABI_TAG;

pub const ELF_NOTE_OS_LINUX: usize = 0;
pub const ELF_NOTE_OS_GNU: usize = 1;
pub const ELF_NOTE_OS_SOLARIS2: usize = 2;
pub const ELF_NOTE_OS_FREEBSD: usize = 3;

pub const NT_GNU_BUILD_ID: usize = 3;
pub const NT_GNU_GOLD_VERSION: usize = 4;

#[repr(C)]
pub struct Elf64_Move {
    pub m_value: Elf64_Xword,
    pub m_info: Elf64_Xword,
    pub m_poffset: Elf64_Xword,
    pub m_repeat: Elf64_Half,
    pub m_stride: Elf64_Half,
}

pub const R_AARCH64_NONE: usize = 0;
pub const R_AARCH64_ABS64: usize = 257;
pub const R_AARCH64_ABS32: usize = 258;
pub const R_AARCH64_ABS16: usize = 259;
pub const R_AARCH64_PREL64: usize = 260;
pub const R_AARCH64_PREL32: usize = 261;
pub const R_AARCH64_PREL16: usize = 262;
pub const R_AARCH64_MOVW_UABS_G0: usize = 263;
pub const R_AARCH64_MOVW_UABS_G0_NC: usize = 264;
pub const R_AARCH64_MOVW_UABS_G1: usize = 265;
pub const R_AARCH64_MOVW_UABS_G1_NC: usize = 266;
pub const R_AARCH64_MOVW_UABS_G2: usize = 267;
pub const R_AARCH64_MOVW_UABS_G2_NC: usize = 268;
pub const R_AARCH64_MOVW_UABS_G3: usize = 269;
pub const R_AARCH64_MOVW_SABS_G0: usize = 270;
pub const R_AARCH64_MOVW_SABS_G1: usize = 271;
pub const R_AARCH64_MOVW_SABS_G2: usize = 272;
pub const R_AARCH64_LD_PREL_LO19: usize = 273;
pub const R_AARCH64_ADR_PREL_LO21: usize = 274;
pub const R_AARCH64_ADR_PREL_PG_HI21: usize = 275;
pub const R_AARCH64_ADR_PREL_PG_HI21_NC: usize = 276;
pub const R_AARCH64_ADD_ABS_LO12_NC: usize = 277;
pub const R_AARCH64_LDST8_ABS_LO12_NC: usize = 278;
pub const R_AARCH64_TSTBR14: usize = 279;
pub const R_AARCH64_CONDBR19: usize = 280;
pub const R_AARCH64_JUMP26: usize = 282;
pub const R_AARCH64_CALL26: usize = 283;
pub const R_AARCH64_LDST16_ABS_LO12_NC: usize = 284;
pub const R_AARCH64_LDST32_ABS_LO12_NC: usize = 285;
pub const R_AARCH64_LDST64_ABS_LO12_NC: usize = 286;
pub const R_AARCH64_MOVW_PREL_G0: usize = 287;
pub const R_AARCH64_MOVW_PREL_G0_NC: usize = 288;
pub const R_AARCH64_MOVW_PREL_G1: usize = 289;
pub const R_AARCH64_MOVW_PREL_G1_NC: usize = 290;
pub const R_AARCH64_MOVW_PREL_G2: usize = 291;
pub const R_AARCH64_MOVW_PREL_G2_NC: usize = 292;
pub const R_AARCH64_MOVW_PREL_G3: usize = 293;
pub const R_AARCH64_LDST128_ABS_LO12_NC: usize = 299;
pub const R_AARCH64_MOVW_GOTOFF_G0: usize = 300;
pub const R_AARCH64_MOVW_GOTOFF_G0_NC: usize = 301;
pub const R_AARCH64_MOVW_GOTOFF_G1: usize = 302;
pub const R_AARCH64_MOVW_GOTOFF_G1_NC: usize = 303;
pub const R_AARCH64_MOVW_GOTOFF_G2: usize = 304;
pub const R_AARCH64_MOVW_GOTOFF_G2_NC: usize = 305;
pub const R_AARCH64_MOVW_GOTOFF_G3: usize = 306;
pub const R_AARCH64_GOTREL64: usize = 307;
pub const R_AARCH64_GOTREL32: usize = 308;
pub const R_AARCH64_GOT_LD_PREL19: usize = 309;
pub const R_AARCH64_LD64_GOTOFF_LO15: usize = 310;
pub const R_AARCH64_ADR_GOT_PAGE: usize = 311;
pub const R_AARCH64_LD64_GOT_LO12_NC: usize = 312;
pub const R_AARCH64_LD64_GOTPAGE_LO15: usize = 313;
pub const R_AARCH64_TLSGD_ADR_PREL21: usize = 512;
pub const R_AARCH64_TLSGD_ADR_PAGE21: usize = 513;
pub const R_AARCH64_TLSGD_ADD_LO12_NC: usize = 514;
pub const R_AARCH64_TLSGD_MOVW_G1: usize = 515;
pub const R_AARCH64_TLSGD_MOVW_G0_NC: usize = 516;
pub const R_AARCH64_TLSLD_ADR_PREL21: usize = 517;
pub const R_AARCH64_TLSLD_ADR_PAGE21: usize = 518;
pub const R_AARCH64_TLSLD_ADD_LO12_NC: usize = 519;
pub const R_AARCH64_TLSLD_MOVW_G1: usize = 520;
pub const R_AARCH64_TLSLD_MOVW_G0_NC: usize = 521;
pub const R_AARCH64_TLSLD_LD_PREL19: usize = 522;
pub const R_AARCH64_TLSLD_MOVW_DTPREL_G2: usize = 523;
pub const R_AARCH64_TLSLD_MOVW_DTPREL_G1: usize = 524;
pub const R_AARCH64_TLSLD_MOVW_DTPREL_G1_NC: usize = 525;
pub const R_AARCH64_TLSLD_MOVW_DTPREL_G0: usize = 526;
pub const R_AARCH64_TLSLD_MOVW_DTPREL_G0_NC: usize = 527;
pub const R_AARCH64_TLSLD_ADD_DTPREL_HI12: usize = 528;
pub const R_AARCH64_TLSLD_ADD_DTPREL_LO12: usize = 529;
pub const R_AARCH64_TLSLD_ADD_DTPREL_LO12_NC: usize = 530;
pub const R_AARCH64_TLSLD_LDST8_DTPREL_LO12: usize = 531;
pub const R_AARCH64_TLSLD_LDST8_DTPREL_LO12_NC: usize = 532;
pub const R_AARCH64_TLSLD_LDST16_DTPREL_LO12: usize = 533;
pub const R_AARCH64_TLSLD_LDST16_DTPREL_LO12_NC: usize = 534;
pub const R_AARCH64_TLSLD_LDST32_DTPREL_LO12: usize = 535;
pub const R_AARCH64_TLSLD_LDST32_DTPREL_LO12_NC: usize = 536;
pub const R_AARCH64_TLSLD_LDST64_DTPREL_LO12: usize = 537;
pub const R_AARCH64_TLSLD_LDST64_DTPREL_LO12_NC: usize = 538;
pub const R_AARCH64_TLSIE_MOVW_GOTTPREL_G1: usize = 539;
pub const R_AARCH64_TLSIE_MOVW_GOTTPREL_G0_NC: usize = 540;
pub const R_AARCH64_TLSIE_ADR_GOTTPREL_PAGE21: usize = 541;
pub const R_AARCH64_TLSIE_LD64_GOTTPREL_LO12_NC: usize = 542;
pub const R_AARCH64_TLSIE_LD_GOTTPREL_PREL19: usize = 543;
pub const R_AARCH64_TLSLE_MOVW_TPREL_G2: usize = 544;
pub const R_AARCH64_TLSLE_MOVW_TPREL_G1: usize = 545;
pub const R_AARCH64_TLSLE_MOVW_TPREL_G1_NC: usize = 546;
pub const R_AARCH64_TLSLE_MOVW_TPREL_G0: usize = 547;
pub const R_AARCH64_TLSLE_MOVW_TPREL_G0_NC: usize = 548;
pub const R_AARCH64_TLSLE_ADD_TPREL_HI12: usize = 549;
pub const R_AARCH64_TLSLE_ADD_TPREL_LO12: usize = 550;
pub const R_AARCH64_TLSLE_ADD_TPREL_LO12_NC: usize = 551;
pub const R_AARCH64_TLSLE_LDST8_TPREL_LO12: usize = 552;
pub const R_AARCH64_TLSLE_LDST8_TPREL_LO12_NC: usize = 553;
pub const R_AARCH64_TLSLE_LDST16_TPREL_LO12: usize = 554;
pub const R_AARCH64_TLSLE_LDST16_TPREL_LO12_NC: usize = 555;
pub const R_AARCH64_TLSLE_LDST32_TPREL_LO12: usize = 556;
pub const R_AARCH64_TLSLE_LDST32_TPREL_LO12_NC: usize = 557;
pub const R_AARCH64_TLSLE_LDST64_TPREL_LO12: usize = 558;
pub const R_AARCH64_TLSLE_LDST64_TPREL_LO12_NC: usize = 559;
pub const R_AARCH64_TLSDESC_LD_PREL19: usize = 560;
pub const R_AARCH64_TLSDESC_ADR_PREL21: usize = 561;
pub const R_AARCH64_TLSDESC_ADR_PAGE21: usize = 562;
pub const R_AARCH64_TLSDESC_LD64_LO12: usize = 563;
pub const R_AARCH64_TLSDESC_ADD_LO12: usize = 564;
pub const R_AARCH64_TLSDESC_OFF_G1: usize = 565;
pub const R_AARCH64_TLSDESC_OFF_G0_NC: usize = 566;
pub const R_AARCH64_TLSDESC_LDR: usize = 567;
pub const R_AARCH64_TLSDESC_ADD: usize = 568;
pub const R_AARCH64_TLSDESC_CALL: usize = 569;
pub const R_AARCH64_TLSLE_LDST128_TPREL_LO12: usize = 570;
pub const R_AARCH64_TLSLE_LDST128_TPREL_LO12_NC: usize = 571;
pub const R_AARCH64_TLSLD_LDST128_DTPREL_LO12: usize = 572;
pub const R_AARCH64_TLSLD_LDST128_DTPREL_LO12_NC: usize = 573;
pub const R_AARCH64_COPY: usize = 1024;
pub const R_AARCH64_GLOB_DAT: usize = 1025;
pub const R_AARCH64_JUMP_SLOT: usize = 1026;
pub const R_AARCH64_RELATIVE: usize = 1027;
pub const R_AARCH64_TLS_DTPMOD64: usize = 1028;
pub const R_AARCH64_TLS_DTPREL64: usize = 1029;
pub const R_AARCH64_TLS_TPREL64: usize = 1030;
pub const R_AARCH64_TLSDESC: usize = 1031;

pub const R_X86_64_NONE: usize = 0;
pub const R_X86_64_64: usize = 1;
pub const R_X86_64_PC32: usize = 2;
pub const R_X86_64_GOT32: usize = 3;
pub const R_X86_64_PLT32: usize = 4;
pub const R_X86_64_COPY: usize = 5;
pub const R_X86_64_GLOB_DAT: usize = 6;
pub const R_X86_64_JUMP_SLOT: usize = 7;
pub const R_X86_64_RELATIVE: usize = 8;
pub const R_X86_64_GOTPCREL: usize = 9;
pub const R_X86_64_32: usize = 10;
pub const R_X86_64_32S: usize = 11;
pub const R_X86_64_16: usize = 12;
pub const R_X86_64_PC16: usize = 13;
pub const R_X86_64_8: usize = 14;
pub const R_X86_64_PC8: usize = 15;
pub const R_X86_64_DTPMOD64: usize = 16;
pub const R_X86_64_DTPOFF64: usize = 17;
pub const R_X86_64_TPOFF64: usize = 18;
pub const R_X86_64_TLSGD: usize = 19;
pub const R_X86_64_TLSLD: usize = 20;
pub const R_X86_64_DTPOFF32: usize = 21;
pub const R_X86_64_GOTTPOFF: usize = 22;
pub const R_X86_64_TPOFF32: usize = 23;
pub const R_X86_64_PC64: usize = 24;
pub const R_X86_64_GOTOFF64: usize = 25;
pub const R_X86_64_GOTPC32: usize = 26;
pub const R_X86_64_GOT64: usize = 27;
pub const R_X86_64_GOTPCREL64: usize = 28;
pub const R_X86_64_GOTPC64: usize = 29;
pub const R_X86_64_GOTPLT64: usize = 30;
pub const R_X86_64_PLTOFF64: usize = 31;
pub const R_X86_64_SIZE32: usize = 32;
pub const R_X86_64_SIZE64: usize = 33;
pub const R_X86_64_GOTPC32_TLSDESC: usize = 34;
pub const R_X86_64_TLSDESC_CALL: usize = 35;
pub const R_X86_64_TLSDESC: usize = 36;
pub const R_X86_64_IRELATIVE: usize = 37;
pub const R_X86_64_RELATIVE64: usize = 38;
pub const R_X86_64_NUM: usize = 39;

#[no_mangle]
pub extern "C" fn stupid_cbindgen_needs_a_function_that_holds_all_elf_structs(
    a: Elf32_Ehdr,
    b: Elf64_Ehdr,
    c: Elf32_Shdr,
    d: Elf64_Shdr,
    e: Elf32_Sym,
    f: Elf64_Sym,
    g: Elf32_Syminfo,
    h: Elf64_Syminfo,
    i: Elf32_Rel,
    j: Elf64_Rel,
    k: Elf32_Rela,
    l: Elf64_Rela,
    m: Elf32_Phdr,
    n: Elf64_Phdr,
    o: Elf32_Dyn,
    p: Elf64_Dyn,
    q: Elf32_Verdef,
    r: Elf64_Verdef,
    s: Elf32_Verdaux,
    t: Elf64_Verdaux,
    u: Elf32_Verneed,
    v: Elf64_Verneed,
    w: Elf64_Vernaux,
    x: Elf64_auxv_t,
    y: Elf32_Nhdr,
    z: Elf64_Nhdr,
    aa: Elf64_Move,
) {
}
