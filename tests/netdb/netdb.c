/* Copyright (C) 1998-2018 Free Software Foundation, Inc.
   This file is part of the GNU C Library.
   Contributed by Andreas Jaeger <aj@suse.de>, 1998.
   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.
   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.
   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, see
   <http://www.gnu.org/licenses/>.  */
/*
  Testing of some network related lookup functions.
  The system databases looked up are:
  - /etc/services
  - /etc/hosts
  - /etc/networks
  - /etc/protocols
  The tests try to be fairly generic and simple so that they work on
  every possible setup (and might therefore not detect some possible
  errors).
*/

#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/param.h>
#include <sys/socket.h>
#include <unistd.h>
#include <errno.h>

#include "test_helpers.h"

int error_count;
static void
output_servent (const char *call, struct servent *sptr)
{
  char **pptr;
  if (sptr == NULL)
    printf ("Call: %s returned NULL\n", call);
  else
    {
      printf ("Call: %s, returned: s_name: %s, s_port: %d, s_proto: %s\n",
              call, sptr->s_name, ntohs(sptr->s_port), sptr->s_proto);
      for (pptr = sptr->s_aliases; *pptr != NULL; pptr++)
        printf ("  alias: %s\n", *pptr);
    }
}
static void
test_services (void)
{
  struct servent *sptr;
  sptr = getservbyname ("domain", "tcp");
  // output_servent ("getservbyname (\"domain\", \"tcp\")", sptr);
  sptr = getservbyname ("domain", "udp");
  // output_servent ("getservbyname (\"domain\", \"udp\")", sptr);
  sptr = getservbyname ("domain", NULL);
  // output_servent ("getservbyname (\"domain\", NULL)", sptr);
  sptr = getservbyname ("not-existant", NULL);
  // output_servent ("getservbyname (\"not-existant\", NULL)", sptr);
  /* This shouldn't return anything.  */
  sptr = getservbyname ("", "");
  // output_servent ("getservbyname (\"\", \"\")", sptr);
  sptr = getservbyname ("", "tcp");
  // output_servent ("getservbyname (\"\", \"tcp\")", sptr);
  sptr = getservbyport (htons(53), "tcp");
  // output_servent ("getservbyport (htons(53), \"tcp\")", sptr);
  sptr = getservbyport (htons(53), NULL);
  // output_servent ("getservbyport (htons(53), NULL)", sptr);
  sptr = getservbyport (htons(1), "udp"); /* shouldn't exist */
  // output_servent ("getservbyport (htons(1), \"udp\")", sptr);
  setservent (0);
  do
    {
      sptr = getservent ();
      //output_servent ("getservent ()", sptr);
    }
  while (sptr != NULL);
  endservent ();
}
static void
output_hostent (const char *call, struct hostent *hptr)
{
  char **pptr;
  char buf[INET6_ADDRSTRLEN];
  if (hptr == NULL)
    printf ("Call: %s returned NULL\n", call);
  else
    {
      printf ("Call: %s returned: name: %s, addr_type: %d\n",
              call, hptr->h_name, hptr->h_addrtype);
      if (hptr->h_aliases)
        for (pptr = hptr->h_aliases; *pptr != NULL; pptr++)
          printf ("  alias: %s\n", *pptr);
      for (pptr = hptr->h_addr_list; *pptr != NULL; pptr++)
        printf ("  ip: %s\n",
                inet_ntop (hptr->h_addrtype, *pptr, buf, sizeof (buf)));
    }
}
static void
test_hosts (void)
{
  struct hostent *hptr1, *hptr2;
  char *name = NULL;
  size_t namelen = 0;
  struct in_addr ip;
  hptr1 = gethostbyname ("localhost");
  hptr2 = gethostbyname ("LocalHost");
  if (hptr1 != NULL || hptr2 != NULL)
    {
      if (hptr1 == NULL)
        {
          printf ("localhost not found - but LocalHost found:-(\n");
          ++error_count;
        }
      else if (hptr2 == NULL)
        {
          printf ("LocalHost not found - but localhost found:-(\n");
          ++error_count;
        }
      else if (strcmp (hptr1->h_name, hptr2->h_name) != 0)
        {
          printf ("localhost and LocalHost have different canoncial name\n");
          printf ("gethostbyname (\"localhost\")->%s\n", hptr1->h_name);
          printf ("gethostbyname (\"LocalHost\")->%s\n", hptr2->h_name);
          ++error_count;
        }
       //else
       //  output_hostent ("gethostbyname(\"localhost\")", hptr1);
    }
  hptr1 = gethostbyname ("127.0.0.1");
  //output_hostent ("gethostbyname (\"127.0.0.1\")", hptr1);
  while (gethostname (name, namelen) < 0 && errno == ENAMETOOLONG)
    {
      namelen += 2;                /* tiny increments to test a lot */
      name = realloc (name, namelen);
    }
  if (gethostname (name, namelen) == 0)
    {
      // printf ("Hostname: %s\n", name);
      if (name != NULL)
        {
          hptr1 = gethostbyname (name);
        //  output_hostent ("gethostbyname (gethostname(...))", hptr1);
        }
    }
  ip.s_addr = htonl (INADDR_LOOPBACK);

  hptr1 = gethostbyaddr ((char *) &ip, sizeof(ip), AF_INET);
  if (hptr1 != NULL)
    {
     // printf ("official name of 127.0.0.1: %s\n", hptr1->h_name);
    }
  sethostent (0);
  do
    {
      hptr1 = gethostent ();
      //output_hostent ("gethostent ()", hptr1);
    }
  while (hptr1 != NULL);
  endhostent ();
  struct hostent* redox = gethostbyname("redox-os.org");
  if (redox == NULL) {
      ++error_count;
  }
  //output_hostent("gethostbyname(\"redox-os.org\")", redox);
  struct in_addr el_goog;
  inet_aton("8.8.4.4", &el_goog);
  struct hostent* google = gethostbyaddr(&el_goog, 4, AF_INET);
  if (google == NULL) {
      ++error_count;
  }
  //output_hostent("gethostbyaddr(\"8.8.4.4\")",google);
}

static void
output_protoent (const char *call, struct protoent *prptr)
{
  char **pptr;
  if (prptr == NULL)
    printf ("Call: %s returned NULL\n", call);
  else
    {
      printf ("Call: %s, returned: p_name: %s, p_proto: %d\n",
              call, prptr->p_name, prptr->p_proto);
      for (pptr = prptr->p_aliases; *pptr != NULL; pptr++)
        printf ("  alias: %s\n", *pptr);
    }
}
static void
test_protocols (void)
{
  struct protoent *prptr;
  prptr = getprotobyname ("ICMP");
 // output_protoent ("getprotobyname (\"ICMP\")", prptr);
  prptr = getprotobynumber (1);
 // output_protoent ("getprotobynumber (1)", prptr);
  setprotoent (0);
  do
    {
      prptr = getprotoent ();
     // output_protoent ("getprotoent ()", prptr);
    }
  while (prptr != NULL);
  endprotoent ();
}
static void
test_network (void)
{
  struct netent *nptr = getnetbyname ("loopback");
  if (nptr != NULL) {
    printf("network name %d", nptr->n_net);
  } else {
    ++error_count;
  }
  do
    {
      nptr = getnetent();
      if (nptr != NULL) {
          printf("network name %s", nptr->n_name);
      }
    }
  while (nptr != NULL);
  setnetent (0);
}
static int
do_test (void)
{
  /*
    setdb ("db");
  */
  test_hosts ();
  test_network ();
  test_protocols ();
  test_services ();
  if (error_count)
    printf ("\n %d errors occurred!\n", error_count);
  else
    printf ("No visible errors occurred!\n");
  return (error_count != 0);
}

int main(void) {
    do_test();
}
