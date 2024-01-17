/* Test of pty.h and forkpty function.
   Copyright (C) 2009-2011 Free Software Foundation, Inc.

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation; either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.  */

/* Written by Simon Josefsson <simon@josefsson.org>, 2009.  */

#include <pty.h>
#include <stdio.h>
#include <sys/wait.h>

int main () {
  int res = 0;
  int amaster = 0;

  res = forkpty (&amaster, NULL, NULL, NULL);
  if (res == 0) {
    printf("This is child process\n");
  } else if (res > 0) {
    printf("This is parent process\n");
    wait(NULL);
  } else {
      printf ("forkpty returned %d\n", res);
      return 1;
  }
  return 0;
}