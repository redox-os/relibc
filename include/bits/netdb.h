#ifndef _BITS_NETDB_H
#define _BITS_NETDB_H

#ifdef __cplusplus
extern "C" {
#endif

#define h_errno (*__h_errno_location())
# define        h_addr  h_addr_list[0] /* Address, for backward compatibility.*/

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* _BITS_NETDB_H */
