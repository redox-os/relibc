// test code found on dbus, currently hangs on redox

#include <sys/epoll.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>

#define _DBUS_ZERO(object) (memset (&(object), '\0', sizeof ((object))))

int
main (void)
{
  struct epoll_event input;
  struct epoll_event output;
  int epfd = epoll_create1 (EPOLL_CLOEXEC);
  
  int pipefds[2];
  pipe(pipefds);
  close(pipefds[1]); 
  int fd = pipefds[0]; 
  
  int ret;

  _DBUS_ZERO (input);

  input.events = EPOLLHUP | EPOLLET;
  ret = epoll_ctl (epfd, EPOLL_CTL_ADD, fd, &input);
  printf ("ctl ADD: %d\n", ret);

  ret = epoll_wait (epfd, &output, 1, -1);
  printf ("wait for HUP, edge-triggered: %d\n", ret);

  ret = epoll_wait (epfd, &output, 1, 100); 
  printf ("wait for HUP again: %d\n", ret);

  input.events = EPOLLHUP;
  ret = epoll_ctl (epfd, EPOLL_CTL_MOD, fd, &input);
  printf ("ctl MOD: %d\n", ret);

  ret = epoll_wait (epfd, &output, 1, -1);
  printf ("wait for HUP: %d\n", ret);

  close(fd);
  close(epfd);
  return 0;
}