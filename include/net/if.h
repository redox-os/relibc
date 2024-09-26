// This is a dummy implementation written originally for compiling libuv

#ifndef _NET_IF_H
#define _NET_IF_H 1

struct if_nameindex {
  unsigned int if_index;
  char *if_name;
};

inline void if_freenameindex(struct if_nameindex *) {}

inline char *if_indextoname(unsigned, char * buf) { return buf; }

inline struct if_nameindex *if_nameindex(void) { return 0; }

inline unsigned if_nametoindex(const char *) { return 1; }

#endif
