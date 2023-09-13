#ifndef _BITS_SYS_SOCKET_H
#define _BITS_SYS_SOCKET_H

struct sockaddr_storage {
	sa_family_t ss_family;
	char __ss_padding[128-sizeof(long)-sizeof(sa_family_t)];
	unsigned long __ss_align;
};

struct ucred {
	pid_t pid;
	uid_t uid;
	gid_t gid;
};

#endif // _BITS_SYS_SOCKET_H
