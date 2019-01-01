#ifndef __MACHINE_ENDIAN_H__

/* TODO: Forcing little endian, if you need a big endian system, fix this { */
#ifndef BIG_ENDIAN
#define BIG_ENDIAN 4321
#endif

#ifndef LITTLE_ENDIAN
#define LITTLE_ENDIAN 1234
#endif

#ifndef BYTE_ORDER
#define BYTE_ORDER LITTLE_ENDIAN
#endif
/* } */

#endif /* __MACHINE_ENDIAN_H__ */
