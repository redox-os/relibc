#ifndef _BITS_NETDB_H
#define _BITS_NETDB_H

#define EAI_BADFLAGS (-1)
#define EAI_NONAME (-2)
#define EAI_AGAIN (-3)
#define EAI_FAIL (-4)
#define EAI_NODATA (-5)
#define EAI_FAMILY (-6)
#define EAI_SOCKTYPE (-7)
#define EAI_SERVICE (-8)
#define EAI_ADDRFAMILY (-9)
#define EAI_MEMORY (-10)
#define EAI_SYSTEM (-11)
#define EAI_OVERFLOW (-12)

# define        h_addr  h_addr_list[0] /* Address, for backward compatibility.*/

#endif /* _BITS_NETDB_H */
