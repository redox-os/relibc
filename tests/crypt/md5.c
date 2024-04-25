/* Copyright (C) 1991-2024 Free Software Foundation, Inc.
   This file is part of the GNU C Library.

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
   <https://www.gnu.org/licenses/>.  */

#include <crypt.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main () {
    const char salt[] = "$1$saltstring";
    char *cp;
    int result = 0;

    cp = crypt ("Hello world!", salt);

    /* MD5 is disabled in FIPS mode.  */
    if (cp) {
        result |= strcmp ("$1$saltstri$YMyguxXMBpd2TEZ.vS/3q1", cp);
        if (!result)
            printf("Success!\n");
    }

    return result;
}